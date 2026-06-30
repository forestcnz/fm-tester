//! HTTP 应用服务
//!
//! 处理 HTTP 请求相关的业务逻辑，协调基础设施 HTTP 客户端和领域服务。
//! 响应时间统计（运行时状态）放在 Application 层。

use chrono::Local;
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

use crate::application::services::WorkspaceDataApplicationService;
use crate::domain::models::{Cookie, FormField, Header, HttpResponse, Variable};
use crate::domain::services::{parse_set_cookie, shell_escape, WorkspaceDataDomainService};
use crate::infrastructure::sse_client::get_sse_client;
use crate::infrastructure::{HttpClientService, RepositoryFactory};

const ONE_MINUTE_MS: u64 = 60 * 1000;

/// 响应时间记录
pub struct ResponseTimeRecord {
    pub timestamp: u64,
    pub time: u64,
}

lazy_static! {
    /// 响应时间统计（运行时状态，放在 Application 层）
    static ref RESPONSE_TIME_STATS: Mutex<HashMap<String, Vec<ResponseTimeRecord>>> =
        Mutex::new(HashMap::new());
}

/// HTTP 日志结构
#[derive(Clone, Serialize)]
pub struct HttpLog {
    #[serde(rename = "logType")]
    pub log_type: String,
    pub timestamp: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// HTTP 应用服务
pub struct HttpApplicationService;

impl HttpApplicationService {
    /// 发送日志事件
    pub fn emit_log(app: &AppHandle, log: HttpLog) {
        if let Err(e) = app.emit("http-log", log) {
            eprintln!("发送日志事件失败: {}", e);
        }
    }

    /// 解析 Set-Cookie header
    pub fn parse_cookie(cookie_str: &str, default_domain: &str) -> Result<Cookie, String> {
        parse_set_cookie(cookie_str, default_domain)
    }

    /// Shell 转义
    pub fn shell_escape(s: &str) -> String {
        shell_escape(s)
    }

    /// 记录响应时间（Application 层状态管理）
    pub fn record_response_time(api_id: &str, response_time: u64) {
        let now = chrono::Local::now().timestamp_millis() as u64;
        let mut stats = RESPONSE_TIME_STATS.lock().unwrap();

        let records = stats.entry(api_id.to_string()).or_default();
        records.push(ResponseTimeRecord {
            timestamp: now,
            time: response_time,
        });

        records.retain(|r| now - r.timestamp < ONE_MINUTE_MS);
    }

    /// 获取平均响应时间（Application 层状态管理）
    pub fn get_average_response_time(api_id: &str) -> Option<u64> {
        let now = chrono::Local::now().timestamp_millis() as u64;
        let mut stats = RESPONSE_TIME_STATS.lock().unwrap();

        if let Some(records) = stats.get_mut(api_id) {
            records.retain(|r| now - r.timestamp < ONE_MINUTE_MS);

            if records.is_empty() {
                return None;
            }

            let sum: u64 = records.iter().map(|r| r.time).sum();
            let avg = sum / records.len() as u64;
            return Some(avg);
        }

        None
    }

    /// 获取激活变量（合并环境变量和集合变量）
    pub fn get_merged_variables(
        workspace_id: &str,
        collection_variables: Option<&Vec<Variable>>,
    ) -> Result<HashMap<String, String>, String> {
        let config = WorkspaceDataApplicationService::read_environments(workspace_id)?;
        let mut variables = WorkspaceDataApplicationService::get_active_variables_map(&config);

        if let Some(coll_vars) = collection_variables {
            for v in coll_vars {
                if v.enabled && !v.key.is_empty() {
                    variables.insert(v.key.clone(), v.value.clone());
                }
            }
        }

        Ok(variables)
    }

    /// 替换变量并收集未定义变量
    pub fn replace_vars(
        text: &str,
        variables: &HashMap<String, String>,
    ) -> (String, HashSet<String>) {
        let result = WorkspaceDataDomainService::replace_variables(text, variables);
        (
            result.text,
            result.undefined_variables.into_iter().collect(),
        )
    }

    /// 替换所有变量（URL、Headers、Body）
    pub fn replace_all_variables(
        url: &str,
        headers: &[Header],
        body: Option<&str>,
        variables: &HashMap<String, String>,
    ) -> (String, Vec<Header>, Option<String>, HashSet<String>) {
        let mut all_undefined_vars: HashSet<String> = HashSet::new();

        let (replaced_url, url_undefined) = Self::replace_vars(url, variables);
        all_undefined_vars.extend(url_undefined);

        let replaced_headers: Vec<Header> = headers
            .iter()
            .map(|h| {
                let (value, header_undefined) = Self::replace_vars(&h.value, variables);
                all_undefined_vars.extend(header_undefined);
                Header {
                    key: h.key.clone(),
                    value,
                    enabled: h.enabled,
                    description: h.description.clone(),
                }
            })
            .collect();

        let replaced_body = body.map(|b| {
            let (text, body_undefined) = Self::replace_vars(b, variables);
            all_undefined_vars.extend(body_undefined);
            text
        });

        (
            replaced_url,
            replaced_headers,
            replaced_body,
            all_undefined_vars,
        )
    }

    /// 验证 URL
    pub fn validate_url(url: &str) -> Result<(), String> {
        let trimmed_url = url.trim();
        if !trimmed_url.starts_with("http://") && !trimmed_url.starts_with("https://") {
            return Err(format!(
                "URL 无效: '{}' 不是完整的 URL。请检查环境变量 baseUrl 是否已定义。",
                trimmed_url
            ));
        }
        Ok(())
    }

    /// 发送未定义变量警告日志
    pub fn emit_undefined_vars_warning(app: &AppHandle, undefined_vars: &HashSet<String>) {
        if !undefined_vars.is_empty() {
            let undefined_list: Vec<String> = undefined_vars.iter().cloned().collect();
            let warning_log = HttpLog {
                log_type: "warning".to_string(),
                timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                message: format!("未定义变量: {}", undefined_list.join(", ")),
                data: Some(serde_json::json!({ "undefinedVariables": undefined_list })),
                error: None,
            };
            Self::emit_log(app, warning_log);
        }
    }

    /// 发送请求日志
    pub fn emit_request_log(
        app: &AppHandle,
        method: &str,
        url: &str,
        headers: &[Header],
        body: Option<&str>,
        body_type: Option<&str>,
    ) {
        let request_data = serde_json::json!({
            "method": method.to_uppercase(),
            "url": url,
            "headers": headers,
            "body": body,
            "bodyType": body_type
        });
        let request_log = HttpLog {
            log_type: "request".to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            message: format!("{} {}", method.to_uppercase(), url),
            data: Some(request_data),
            error: None,
        };
        Self::emit_log(app, request_log);
    }

    /// 发送错误日志
    pub fn emit_error_log(app: &AppHandle, message: &str, error: Option<&str>) {
        let err_log = HttpLog {
            log_type: "error".to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            message: message.to_string(),
            data: None,
            error: error.map(|s| s.to_string()),
        };
        Self::emit_log(app, err_log);
    }

    /// 获取超时设置（毫秒）
    pub fn get_timeout(_workspace_id: &str) -> Result<u64, String> {
        let settings_repo = crate::infrastructure::RepositoryFactory::get_app_config_repository();
        let config = settings_repo.read()?;
        Ok(config.settings.request_timeout * 1000)
    }

    /// 发送 HTTP 请求
    pub async fn send_request(
        app: AppHandle,
        method: String,
        url: String,
        headers: Vec<Header>,
        body: Option<String>,
        body_type: Option<String>,
        form_fields: Option<Vec<FormField>>,
        workspace_id: String,
        api_id: Option<String>,
        _api_name: Option<String>,
        collection_variables: Option<Vec<Variable>>,
    ) -> Result<HttpResponse, String> {
        let variables = Self::get_merged_variables(&workspace_id, collection_variables.as_ref())?;

        let (replaced_url, replaced_headers, replaced_body, undefined_vars) =
            Self::replace_all_variables(&url, &headers, body.as_deref(), &variables);

        // 去除URL前后空格
        let trimmed_url = replaced_url.trim();

        tracing::info!(
            "HTTP 请求: {} {} (api_id={})",
            method,
            trimmed_url,
            api_id.as_deref().unwrap_or("-")
        );

        if let Err(e) = Self::validate_url(trimmed_url) {
            Self::emit_error_log(&app, &e, Some("URL 无效"));
            return Err(e);
        }

        Self::emit_undefined_vars_warning(&app, &undefined_vars);
        Self::emit_request_log(
            &app,
            &method,
            trimmed_url,
            &replaced_headers,
            replaced_body.as_deref(),
            body_type.as_deref(),
        );

        let timeout = Self::get_timeout(&workspace_id)?;

        // 记录端到端开始时间
        let overall_start = std::time::Instant::now();

        // 发送请求（分阶段计时）
        let response = HttpClientService::send_with_timing(
            &method,
            trimmed_url,
            replaced_headers.clone(),
            replaced_body.clone(),
            body_type.clone(),
            form_fields.clone(),
            timeout,
        )
        .await;

        match response {
            Ok(timing_resp) => {
                // 检查 Content-Type 是否为 SSE
                let content_type = timing_resp
                    .headers
                    .get("content-type")
                    .map(|v| v.as_str())
                    .unwrap_or("");

                if content_type.contains("text/event-stream") {
                    // SSE 流式响应：透传响应信息
                    let status = timing_resp.status;
                    let status_text = timing_resp.status_text.clone();
                    let resp_headers = timing_resp.headers.clone();
                    let final_url = timing_resp.final_url.clone();

                    app.emit(
                        "sse-response-info",
                        serde_json::json!({
                            "status": status,
                            "statusText": status_text,
                            "headers": resp_headers,
                            "resolvedUrl": final_url,
                        }),
                    )
                    .ok();

                    // SSE 流式响应，调用 SSE 客户端处理
                    Self::emit_request_log(
                        &app,
                        &method,
                        &replaced_url,
                        &replaced_headers,
                        replaced_body.as_deref(),
                        body_type.as_deref(),
                    );

                    let sse_client = get_sse_client();
                    sse_client.process_sse_stream(app, timing_resp.body).await?;

                    // SSE 流处理完成，返回空响应（前端通过事件接收数据）
                    return Err("SSE_STREAM".to_string());
                }

                // 非 SSE：读取完整响应体，计下载耗时
                let download_start = std::time::Instant::now();
                let body_bytes = hyper::body::to_bytes(timing_resp.body)
                    .await
                    .map_err(|e| format!("读取响应体失败: {}", e))?;
                let download_ms = download_start.elapsed().as_millis() as u64;

                let mut timing = timing_resp.timing;
                timing.download_ms = download_ms;
                timing.total_ms = overall_start.elapsed().as_millis() as u64;

                let body = String::from_utf8_lossy(&body_bytes).to_string();
                let size = body.len() as u64;
                let elapsed_ms = timing.total_ms;

                let mut http_response = HttpResponse {
                    status: timing_resp.status,
                    status_text: timing_resp.status_text,
                    headers: timing_resp.headers,
                    body,
                    time: elapsed_ms,
                    size,
                    resolved_url: timing_resp.final_url,
                    resolved_headers: replaced_headers.clone(),
                    avg_time: None,
                    timing: Some(timing),
                };

                if let Some(ref id) = api_id {
                    Self::record_response_time(id, http_response.time);
                    http_response.avg_time = Self::get_average_response_time(id);
                }

                let cookie_repo = RepositoryFactory::get_workspace_data_repository();
                let default_domain = Self::extract_domain(&replaced_url);

                if let Some(cookie_header) = http_response.headers.get("set-cookie") {
                    if let Ok(cookie) = Self::parse_cookie(cookie_header, &default_domain) {
                        cookie_repo
                            .add_or_update_cookie(&workspace_id, &cookie)
                            .ok();
                    }
                }

                for (key, value) in &http_response.headers {
                    if key.to_lowercase() == "set-cookie" {
                        if let Ok(cookie) = Self::parse_cookie(value, &default_domain) {
                            cookie_repo
                                .add_or_update_cookie(&workspace_id, &cookie)
                                .ok();
                        }
                    }
                }

                let response_data = serde_json::json!({
                    "url": http_response.resolved_url,
                    "status": http_response.status,
                    "statusText": http_response.status_text,
                    "headers": http_response.headers,
                    "body": http_response.body,
                    "time": http_response.time,
                    "size": http_response.size
                });
                let response_log = HttpLog {
                    log_type: "response".to_string(),
                    timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    message: format!(
                        "{} {} {} - {}ms",
                        method.to_uppercase(),
                        http_response.status,
                        http_response.resolved_url,
                        http_response.time
                    ),
                    data: Some(response_data),
                    error: None,
                };
                Self::emit_log(&app, response_log);

                Ok(http_response)
            }
            Err(e) => {
                // SSE 流返回特殊错误，不记录错误日志
                if e == "SSE_STREAM" {
                    return Err(e);
                }
                Self::emit_error_log(&app, &e, None);
                Err(e)
            }
        }
    }

    /// 从 URL 提取域名
    fn extract_domain(url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            parsed.host_str().unwrap_or("localhost").to_string()
        } else {
            "localhost".to_string()
        }
    }
}

impl Default for HttpApplicationService {
    fn default() -> Self {
        Self
    }
}
