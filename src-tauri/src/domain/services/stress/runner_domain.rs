use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::domain::models::{
    Cookie, FailedRequest, Header, HistoryPoint, ScriptExecutionContext, ScriptInfo,
    ScriptRequestContext, ScriptResponseContext, StressTestConfig, StressTestProgress,
    StressTestResult,
};
use crate::domain::services::WorkspaceDataDomainService;
use crate::infrastructure::JsRuntimeExecutor;
use url::Url;

/// 后置脚本执行结果
struct PostScriptResult {
    /// 脚本是否成功执行
    success: bool,
    /// 所有断言是否通过
    all_tests_passed: bool,
    /// 测试结果列表
    test_results: Vec<crate::domain::models::ScriptTestResult>,
    /// 错误信息
    error: Option<String>,
}

/// 运行时统计
pub struct RuntimeStats {
    pub spawned: AtomicU64,
    pub completed: AtomicU64,
    pub successful: AtomicU64,
    pub failed: AtomicU64,
    pub current_running: AtomicU32,
    pub times: Mutex<Vec<u64>>,
    // TOML requires all map keys to be strings, so we use String instead of u16
    pub status_codes: Mutex<HashMap<String, u64>>,
    pub errors: Mutex<HashMap<String, u64>>,
    pub failed_request_details: Mutex<Vec<FailedRequest>>,
    pub history: Mutex<Vec<HistoryPoint>>,
    pub last_recorded_second: AtomicU64,
}

impl RuntimeStats {
    pub fn new() -> Self {
        Self {
            spawned: AtomicU64::new(0),
            completed: AtomicU64::new(0),
            successful: AtomicU64::new(0),
            failed: AtomicU64::new(0),
            current_running: AtomicU32::new(0),
            times: Mutex::new(Vec::new()),
            status_codes: Mutex::new(HashMap::new()),
            errors: Mutex::new(HashMap::new()),
            failed_request_details: Mutex::new(Vec::new()),
            history: Mutex::new(Vec::new()),
            last_recorded_second: AtomicU64::new(0),
        }
    }
}

/// 压测运行器（核心逻辑，不含 Tauri 事件）
/// 变量映射由调用者（interface层）传入，符合DDD分层原则
pub struct StressTestRunnerCore {
    pub config: StressTestConfig,
    pub stats: Arc<RuntimeStats>,
    pub stop_signal: Arc<AtomicBool>,
    pub start_time: Instant,
    pub variables: HashMap<String, String>,
    pub cookies: Vec<Cookie>,
    /// 后置脚本链（已加载）
    pub post_scripts: Vec<ScriptInfo>,
    /// 共享 HTTP 客户端（禁用连接池，模拟多设备并发场景）
    pub http_client: Arc<reqwest::Client>,
}

impl StressTestRunnerCore {
    pub fn new(
        config: StressTestConfig,
        variables: HashMap<String, String>,
        cookies: Vec<Cookie>,
        post_scripts: Vec<ScriptInfo>,
    ) -> Self {
        // 禁用连接池，每个请求独立建立连接，模拟多设备并发场景。
        // 不在此处设置全局 timeout，改由每次 RequestBuilder::timeout 按配置控制。
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(0)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            config,
            stats: Arc::new(RuntimeStats::new()),
            stop_signal: Arc::new(AtomicBool::new(false)),
            start_time: Instant::now(),
            variables,
            cookies,
            post_scripts,
            http_client: Arc::new(client),
        }
    }

    /// 停止压测
    pub fn stop(&self) {
        self.stop_signal.store(true, Ordering::SeqCst);
    }

    /// 检查 Cookie 是否匹配指定 host（基于 RFC 6265 域匹配规则）
    /// - host 等于 cookie.domain 视为同域
    /// - host 是 cookie.domain 的子域（如 a.example.com 匹配 example.com）视为匹配
    /// - 反过来（cookie.domain 比 host 更长）不算匹配
    fn cookie_matches_domain(cookie: &Cookie, host: &str) -> bool {
        let cookie_domain = cookie.domain.trim_start_matches('.');
        if cookie_domain.is_empty() || host.is_empty() {
            return false;
        }
        // 大小写不敏感比较（域名不区分大小写）
        let host_lower = host.to_lowercase();
        let domain_lower = cookie_domain.to_lowercase();
        host_lower == domain_lower || host_lower.ends_with(&format!(".{}", domain_lower))
    }

    /// 发送单个请求
    pub async fn send_single_request(&self) -> (bool, Option<u16>, Option<String>, u64) {
        // 使用传入的变量映射（由interface层提供）
        let mut variables = self.variables.clone();
        if let Some(ref coll_vars) = self.config.collection_variables {
            for v in coll_vars {
                if v.enabled && !v.key.is_empty() {
                    variables.insert(v.key.clone(), v.value.clone());
                }
            }
        }

        // 替换 URL
        let url_result =
            WorkspaceDataDomainService::replace_variables(&self.config.url, &variables);
        let replaced_url = url_result.text;

        // 验证 URL
        if !replaced_url.starts_with("http://") && !replaced_url.starts_with("https://") {
            return (false, None, Some("URL 无效".to_string()), 0);
        }

        // 替换 Headers
        let replaced_headers: Vec<Header> = self
            .config
            .headers
            .iter()
            .map(|h| {
                let value_result =
                    WorkspaceDataDomainService::replace_variables(&h.value, &variables);
                Header {
                    key: h.key.clone(),
                    value: value_result.text,
                    enabled: h.enabled,
                    description: h.description.clone(),
                }
            })
            .collect();

        // 替换 Body
        let replaced_body = self.config.body.as_ref().map(|b| {
            let body_result = WorkspaceDataDomainService::replace_variables(b, &variables);
            body_result.text
        });

        // 单次请求超时（不破坏连接池复用）
        let request_timeout = Duration::from_millis(self.config.timeout_ms);

        // 构建请求（复用共享 client）
        let mut request = match self.config.method.to_uppercase().as_str() {
            "GET" => self.http_client.get(&replaced_url).timeout(request_timeout),
            "POST" => self
                .http_client
                .post(&replaced_url)
                .timeout(request_timeout),
            "PUT" => self.http_client.put(&replaced_url).timeout(request_timeout),
            "DELETE" => self
                .http_client
                .delete(&replaced_url)
                .timeout(request_timeout),
            "PATCH" => self
                .http_client
                .patch(&replaced_url)
                .timeout(request_timeout),
            "HEAD" => self
                .http_client
                .head(&replaced_url)
                .timeout(request_timeout),
            "OPTIONS" => self
                .http_client
                .request(reqwest::Method::OPTIONS, &replaced_url)
                .timeout(request_timeout),
            _ => {
                return (false, None, Some("不支持的 HTTP 方法".to_string()), 0);
            }
        };

        // 获取 domain 并携带 cookies
        let parsed_url = Url::parse(&replaced_url).ok();
        let domain = parsed_url.as_ref().and_then(|u| u.host_str()).unwrap_or("");

        // 使用传入的 cookies（由调用者读取）
        // 按 RFC 6265 域匹配规则筛选后，合并为单个 Cookie 头（reqwest 同名 header 是替换语义，
        // 多次 .header("Cookie", ...) 会只保留最后一个）
        let matching_cookies: Vec<String> = self
            .cookies
            .iter()
            .filter(|c| Self::cookie_matches_domain(c, domain))
            .map(|c| format!("{}={}", c.name, c.value))
            .collect();
        if !matching_cookies.is_empty() {
            request = request.header("Cookie", matching_cookies.join("; "));
        }

        // 处理 Content-Type 跳过逻辑
        let body_type = self
            .config
            .body_type
            .clone()
            .unwrap_or_else(|| "raw".to_string());
        let skip_content_type = body_type == "form-data";

        // 添加 Headers
        for header in &replaced_headers {
            if header.enabled {
                if skip_content_type && header.key.to_lowercase() == "content-type" {
                    continue;
                }
                request = request.header(&header.key, &header.value);
            }
        }

        // 处理请求体
        if self.config.method != "GET" && self.config.method != "HEAD" {
            match body_type.as_str() {
                "form-data" => {
                    // 文件上传需要在interface层预处理，domain层不直接读取文件
                    // 这里只处理文本字段
                    if let Some(ref fields) = self.config.form_fields {
                        let mut form = reqwest::multipart::Form::new();
                        for field in fields {
                            if !field.enabled || field.key.is_empty() {
                                continue;
                            }
                            // 文件类型需要 interface 层预处理，暂时跳过
                            // TODO: P1 任务中完整重构 HTTP 发送逻辑
                            if field.field_type == "text" {
                                let replaced_value = WorkspaceDataDomainService::replace_variables(
                                    &field.value,
                                    &variables,
                                );
                                form = form.text(field.key.clone(), replaced_value.text);
                            }
                        }
                        request = request.multipart(form);
                    }
                }
                _ => {
                    if let Some(ref b) = replaced_body {
                        request = request.body(b.clone());
                    }
                }
            }
        }

        let start = Instant::now();
        let result = request.send().await;
        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Ok(response) => {
                let status = response.status().as_u16();
                let response_headers: HashMap<String, String> = response
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();

                let response_body = response.text().await.unwrap_or_default();

                // 执行后置脚本（如果有）
                if !self.post_scripts.is_empty() {
                    let script_result = self
                        .execute_post_scripts(
                            &replaced_url,
                            status,
                            &response_body,
                            &response_headers,
                            elapsed,
                            &variables,
                        )
                        .await;

                    // 根据脚本断言结果判断成功
                    if script_result.success && script_result.all_tests_passed {
                        return (true, Some(status), None, elapsed);
                    }

                    // 有断言失败或脚本错误
                    let error_msg = if !script_result.all_tests_passed {
                        let failed_tests: Vec<String> = script_result
                            .test_results
                            .iter()
                            .filter(|t| !t.passed)
                            .map(|t| t.name.clone())
                            .collect();
                        format!("断言失败: {}", failed_tests.join(", "))
                    } else if let Some(err) = script_result.error {
                        format!("脚本执行失败: {}", err)
                    } else {
                        "后置脚本执行异常".to_string()
                    };

                    return (false, Some(status), Some(error_msg), elapsed);
                }

                // 无后置脚本，使用 HTTP 状态码判断成功
                let http_success = (200..400).contains(&status);
                (http_success, Some(status), None, elapsed)
            }
            Err(e) => {
                let error_msg = if e.is_timeout() {
                    "请求超时".to_string()
                } else {
                    e.to_string()
                };
                (false, None, Some(error_msg), elapsed)
            }
        }
    }

    /// 执行后置脚本链
    async fn execute_post_scripts(
        &self,
        url: &str,
        status: u16,
        body: &str,
        headers: &HashMap<String, String>,
        time_ms: u64,
        variables: &HashMap<String, String>,
    ) -> PostScriptResult {
        let mut env_vars = variables.clone();
        let mut coll_vars = HashMap::new();
        let mut all_test_results = Vec::new();
        let mut has_error = false;
        let mut error_msg = None;

        // 构建请求上下文
        let request_ctx = ScriptRequestContext {
            url: url.to_string(),
            method: self.config.method.clone(),
            headers: self.config.headers.clone(),
            params: self.config.params.clone(),
            body: self.config.body.clone(),
        };

        // 构建响应上下文
        let response_ctx = ScriptResponseContext {
            status,
            status_text: "".to_string(),
            headers: headers.clone(),
            body: body.to_string(),
            time: time_ms,
            size: body.len() as u64,
        };

        for script in &self.post_scripts {
            let context = ScriptExecutionContext {
                environment_variables: env_vars.clone(),
                collection_variables: coll_vars.clone(),
                all_collection_variables: HashMap::new(), // 压测不需要按集合分组
                target_collection_id: None,
                target_environment_id: None,
                is_api_script: false,
                parent_collection_id: None,
                request: request_ctx.clone(),
                response: Some(response_ctx.clone()),
            };

            let result =
                JsRuntimeExecutor::execute_script(&script.content, &context, &script.source).await;

            match result {
                Ok(exec_result) => {
                    all_test_results.extend(exec_result.test_results.clone());

                    if !exec_result.success {
                        has_error = true;
                        error_msg = exec_result.error.clone();
                        break;
                    }

                    env_vars = exec_result.modified_environment_vars;
                    coll_vars = exec_result.modified_collection_vars;
                }
                Err(e) => {
                    has_error = true;
                    error_msg = Some(e);
                    break;
                }
            }
        }

        // 检查所有断言是否通过
        let all_passed = all_test_results.iter().all(|t| t.passed);

        PostScriptResult {
            success: !has_error,
            all_tests_passed: all_passed,
            test_results: all_test_results,
            error: error_msg,
        }
    }

    /// 计算统计结果
    pub async fn calculate_result(&self) -> StressTestResult {
        let total_time_ms = self.start_time.elapsed().as_millis() as u64;
        let total_requests = self.stats.completed.load(Ordering::SeqCst);
        let successful = self.stats.successful.load(Ordering::SeqCst);
        let failed = self.stats.failed.load(Ordering::SeqCst);

        let times = self.stats.times.lock().await.clone();
        let status_codes = self.stats.status_codes.lock().await.clone();
        let errors = self.stats.errors.lock().await.clone();
        let failed_request_details = self.stats.failed_request_details.lock().await.clone();
        let history = self.stats.history.lock().await.clone();

        // 计算百分位数
        let (p50, p90, p95, p99, min, max, avg) = calculate_percentiles(&times);

        // 计算 QPS
        let qps = if total_time_ms > 0 {
            (total_requests as f64) * 1000.0 / (total_time_ms as f64)
        } else {
            0.0
        };

        // 计算成功率
        let success_rate = if total_requests > 0 {
            (successful as f64) / (total_requests as f64)
        } else {
            0.0
        };

        StressTestResult {
            id: self.config.id.clone(),
            api_id: self.config.api_id.clone(),
            config: self.config.clone(),
            start_time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            end_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
            total_requests,
            successful_requests: successful,
            failed_requests: failed,
            total_time_ms,
            qps,
            avg_time_ms: avg,
            min_time_ms: min,
            max_time_ms: max,
            p50_time_ms: p50,
            p90_time_ms: p90,
            p95_time_ms: p95,
            p99_time_ms: p99,
            success_rate,
            status_distribution: status_codes,
            error_distribution: errors,
            failed_request_details,
            history,
        }
    }

    /// 获取当前进度（不含事件发送）
    pub async fn get_progress(&self) -> StressTestProgress {
        let completed = self.stats.completed.load(Ordering::SeqCst);
        let successful = self.stats.successful.load(Ordering::SeqCst);
        let failed = self.stats.failed.load(Ordering::SeqCst);
        let elapsed_seconds = self.start_time.elapsed().as_secs() as u32;

        let times = self.stats.times.lock().await;
        let avg_time = if times.is_empty() {
            0.0
        } else {
            times.iter().sum::<u64>() as f64 / times.len() as f64
        };

        let current_qps = if elapsed_seconds > 0 {
            (completed as f64) / (elapsed_seconds as f64)
        } else {
            0.0
        };

        let history = self.stats.history.lock().await.clone();
        let is_running = !self.stop_signal.load(Ordering::SeqCst);

        StressTestProgress {
            id: self.config.id.clone(),
            elapsed_seconds,
            completed_requests: completed,
            current_qps,
            current_avg_time_ms: avg_time,
            successful_requests: successful,
            failed_requests: failed,
            current_concurrent: self.stats.current_running.load(Ordering::SeqCst),
            is_running,
            history,
        }
    }

    /// 记录历史数据点
    pub async fn record_history(&self, elapsed_seconds: u32) {
        let last_second = self.stats.last_recorded_second.load(Ordering::SeqCst);
        if elapsed_seconds as u64 > last_second {
            self.stats
                .last_recorded_second
                .store(elapsed_seconds as u64, Ordering::SeqCst);

            let completed = self.stats.completed.load(Ordering::SeqCst);
            let successful = self.stats.successful.load(Ordering::SeqCst);
            let failed = self.stats.failed.load(Ordering::SeqCst);
            let spawned = self.stats.spawned.load(Ordering::SeqCst);

            let times = self.stats.times.lock().await;
            let avg_time = if times.is_empty() {
                0.0
            } else {
                times.iter().sum::<u64>() as f64 / times.len() as f64
            };

            let current_qps = if elapsed_seconds > 0 {
                (completed as f64) / (elapsed_seconds as f64)
            } else {
                0.0
            };

            let mut history = self.stats.history.lock().await;
            history.push(HistoryPoint {
                second: elapsed_seconds,
                qps: current_qps,
                avg_time_ms: avg_time,
                successful,
                failed,
                requests: spawned,
                concurrent: self.stats.current_running.load(Ordering::SeqCst),
            });
        }
    }
}

/// 计算百分位数
pub fn calculate_percentiles(times: &[u64]) -> (f64, f64, f64, f64, u64, u64, f64) {
    if times.is_empty() {
        return (0.0, 0.0, 0.0, 0.0, 0, 0, 0.0);
    }

    let mut sorted = times.to_vec();
    sorted.sort();

    let len = sorted.len();
    let avg = sorted.iter().sum::<u64>() as f64 / len as f64;
    let min = sorted[0];
    let max = sorted[len - 1];

    let p50 = percentile(&sorted, 50.0);
    let p90 = percentile(&sorted, 90.0);
    let p95 = percentile(&sorted, 95.0);
    let p99 = percentile(&sorted, 99.0);

    (p50, p90, p95, p99, min, max, avg)
}

fn percentile(sorted: &[u64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let index = (sorted.len() as f64 * p / 100.0).ceil() as usize;
    sorted[index.min(sorted.len()) - 1] as f64
}
