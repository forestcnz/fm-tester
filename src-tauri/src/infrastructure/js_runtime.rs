//! JavaScript Runtime 执行器
//!
//! 使用 rquickjs 提供沙箱化的 JavaScript 执行环境。

use rquickjs::{CaughtError, Context, Ctx, Function, Object, Runtime};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::domain::models::{
    Header, Param, ScriptExecutionContext, ScriptExecutionResult, ScriptLog, ScriptRequestContext,
    ScriptResponseContext, ScriptTestResult,
};

/// 沙箱配置
pub struct SandboxConfig {
    /// 内存限制（字节）
    pub memory_limit: usize,
    /// CPU 执行超时（毫秒），0 表示禁用
    pub cpu_timeout_ms: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        SandboxConfig {
            memory_limit: 10 * 1024 * 1024, // 10MB
            cpu_timeout_ms: 300_000,        // 5 分钟，防止死循环永久占用 blocking 池
        }
    }
}

/// JavaScript Runtime 执行器
pub struct JsRuntimeExecutor;

impl JsRuntimeExecutor {
    /// 创建沙箱化的 JS runtime
    pub fn create_sandboxed_runtime(config: &SandboxConfig) -> Runtime {
        let rt = Runtime::new().expect("Failed to create runtime");
        rt.set_memory_limit(config.memory_limit);

        // CPU 超时保护：脚本死循环时由 interrupt handler 中断
        if config.cpu_timeout_ms > 0 {
            let timeout = Duration::from_millis(config.cpu_timeout_ms);
            let start = Instant::now();
            // rquickjs 的 InterruptHandler 语义：闭包返回 true 表示“中断执行”，返回 false 表示“继续”。
            // （见 rquickjs-core raw.rs 中 should_interrupt 直接透传给 QuickJS，quickjs.c 中
            //   interrupt_handler 返回非零值时抛出不可捕获的 "interrupted" 异常）
            // 因此只有在超时后才返回 true 触发中断。
            rt.set_interrupt_handler(Some(Box::new(move || start.elapsed() >= timeout)));
        }

        rt
    }

    /// 执行单个脚本（异步包装）
    pub async fn execute_script(
        code: &str,
        context: &ScriptExecutionContext,
        source: &str,
    ) -> Result<ScriptExecutionResult, String> {
        if code.trim().is_empty() {
            return Ok(ScriptExecutionResult {
                success: true,
                modified_environment_vars: context.environment_variables.clone(),
                modified_target_environment_vars: None,
                target_environment_id: context.target_environment_id.clone(),
                modified_collection_vars: context.collection_variables.clone(),
                modified_target_collection_vars: None,
                target_collection_id: context.target_collection_id.clone(),
                modified_request: Some(context.request.clone()),
                test_results: Vec::new(),
                logs: Vec::new(),
                error: None,
                error_source: None,
            });
        }

        let code = code.to_string();
        let context = context.clone();
        let source_str = source.to_string();
        let config = SandboxConfig::default();

        // 外层兜底超时：即便 quickjs interrupt handler 因故未触发，
        // tokio timeout 也能让 await 返回，避免永久占用 blocking 池
        let outer_timeout = Duration::from_millis(config.cpu_timeout_ms.saturating_add(5_000));
        // 提前记录（move 进闭包后无法再借用）
        let timeout_ms_for_msg = config.cpu_timeout_ms;
        let source_for_msg = source_str.clone();

        let join_handle = tokio::task::spawn_blocking(move || {
            Self::execute_script_sync(&code, &context, &source_str, &config)
        });

        let result = tokio::time::timeout(outer_timeout, join_handle).await;

        match result {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => Err(format!("Script execution failed: {}", e)),
            Err(_) => Err(format!(
                "Script execution timed out after {} ms in {}",
                timeout_ms_for_msg, source_for_msg
            )),
        }
    }

    /// 执行单个脚本（同步）
    fn execute_script_sync(
        code: &str,
        context: &ScriptExecutionContext,
        source: &str,
        config: &SandboxConfig,
    ) -> Result<ScriptExecutionResult, String> {
        let rt = Self::create_sandboxed_runtime(config);
        let ctx = Context::full(&rt).expect("Failed to create context");

        let logs: Arc<Mutex<Vec<ScriptLog>>> = Arc::new(Mutex::new(Vec::new()));
        let test_results: Arc<Mutex<Vec<ScriptTestResult>>> = Arc::new(Mutex::new(Vec::new()));
        // 所有环境变量（用于传递）
        let env_vars: Arc<Mutex<HashMap<String, String>>> =
            Arc::new(Mutex::new(context.environment_variables.clone()));
        // 目标环境变量（用于环境脚本操作自己的环境）
        let target_env_vars: Arc<Mutex<HashMap<String, String>>> = {
            let vars = if context.target_environment_id.is_some() {
                Some(context.environment_variables.clone())
            } else {
                None
            };
            Arc::new(Mutex::new(vars.unwrap_or_default()))
        };
        // 所有集合变量（用于传递）
        let coll_vars: Arc<Mutex<HashMap<String, String>>> =
            Arc::new(Mutex::new(context.collection_variables.clone()));
        // 目标集合变量（用于集合脚本操作自己的集合）
        let target_coll_vars: Arc<Mutex<HashMap<String, String>>> = {
            // 确定目标集合 ID：集合脚本用 target_collection_id，API脚本用 parent_collection_id
            let target_id = if context.is_api_script {
                context.parent_collection_id.clone()
            } else {
                context.target_collection_id.clone()
            };
            let vars = target_id
                .clone()
                .and_then(|id| context.all_collection_variables.get(&id).cloned());
            Arc::new(Mutex::new(vars.unwrap_or_default()))
        };
        let target_collection_id = if context.is_api_script {
            context.parent_collection_id.clone()
        } else {
            context.target_collection_id.clone()
        };
        let request_state: Arc<Mutex<ScriptRequestContext>> =
            Arc::new(Mutex::new(context.request.clone()));
        let has_response = context.response.is_some();
        let source_str = source.to_string();

        let mut execution_error: Option<String> = None;

        ctx.with(|ctx| {
            // 在一个闭包中设置所有 API，避免生命周期问题
            let result = Self::setup_fm_and_execute(
                ctx,
                code,
                env_vars.clone(),
                target_env_vars.clone(),
                context.target_environment_id.clone(),
                coll_vars.clone(),
                target_coll_vars.clone(),
                target_collection_id.clone(),
                request_state.clone(),
                context.response.clone(),
                logs.clone(),
                test_results.clone(),
                source_str.clone(),
                has_response,
            );

            if let Err(e) = result {
                execution_error = Some(e);
            }
        });

        if let Some(error) = execution_error {
            Ok(ScriptExecutionResult {
                success: false,
                modified_environment_vars: env_vars.lock().unwrap().clone(),
                modified_target_environment_vars: Some(target_env_vars.lock().unwrap().clone()),
                target_environment_id: context.target_environment_id.clone(),
                modified_collection_vars: coll_vars.lock().unwrap().clone(),
                modified_target_collection_vars: Some(target_coll_vars.lock().unwrap().clone()),
                target_collection_id,
                modified_request: Some(request_state.lock().unwrap().clone()),
                test_results: test_results.lock().unwrap().clone(),
                logs: logs.lock().unwrap().clone(),
                error: Some(error),
                error_source: Some(source.to_string()),
            })
        } else {
            Ok(ScriptExecutionResult {
                success: true,
                modified_environment_vars: env_vars.lock().unwrap().clone(),
                modified_target_environment_vars: Some(target_env_vars.lock().unwrap().clone()),
                target_environment_id: context.target_environment_id.clone(),
                modified_collection_vars: coll_vars.lock().unwrap().clone(),
                modified_target_collection_vars: Some(target_coll_vars.lock().unwrap().clone()),
                target_collection_id,
                modified_request: Some(request_state.lock().unwrap().clone()),
                test_results: test_results.lock().unwrap().clone(),
                logs: logs.lock().unwrap().clone(),
                error: None,
                error_source: None,
            })
        }
    }

    /// 设置 fm API 并执行脚本（统一处理生命周期）
    fn setup_fm_and_execute<'js>(
        ctx: Ctx<'js>,
        code: &str,
        env_vars: Arc<Mutex<HashMap<String, String>>>,
        target_env_vars: Arc<Mutex<HashMap<String, String>>>,
        target_environment_id: Option<String>,
        coll_vars: Arc<Mutex<HashMap<String, String>>>,
        target_coll_vars: Arc<Mutex<HashMap<String, String>>>,
        target_collection_id: Option<String>,
        request_state: Arc<Mutex<ScriptRequestContext>>,
        response: Option<ScriptResponseContext>,
        logs: Arc<Mutex<Vec<ScriptLog>>>,
        test_results: Arc<Mutex<Vec<ScriptTestResult>>>,
        source: String,
        has_response: bool,
    ) -> Result<(), String> {
        let globals = ctx.globals();

        // 创建 fm 对象
        let fm = Object::new(ctx.clone()).map_err(|e| format!("Failed to create fm: {}", e))?;

        // ===== environment API =====
        // get 操作所有环境变量，set/remove 操作目标环境变量
        let env_obj =
            Object::new(ctx.clone()).map_err(|e| format!("Failed to create env_obj: {}", e))?;

        env_obj
            .set(
                "get",
                Function::new(ctx.clone(), {
                    let vars = env_vars.clone();
                    move |key: String| vars.lock().unwrap().get(&key).cloned()
                }),
            )
            .map_err(|e| format!("Failed to set env.get: {}", e))?;

        // set 同时更新所有变量和目标变量
        env_obj
            .set(
                "set",
                Function::new(ctx.clone(), {
                    let vars = env_vars.clone();
                    let target_vars = target_env_vars.clone();
                    let has_target = target_environment_id.is_some();
                    move |key: String, value: String| {
                        vars.lock().unwrap().insert(key.clone(), value.clone());
                        if has_target {
                            target_vars.lock().unwrap().insert(key, value);
                        }
                    }
                }),
            )
            .map_err(|e| format!("Failed to set env.set: {}", e))?;

        // remove 同时从所有变量和目标变量中删除
        env_obj
            .set(
                "remove",
                Function::new(ctx.clone(), {
                    let vars = env_vars.clone();
                    let target_vars = target_env_vars.clone();
                    let has_target = target_environment_id.is_some();
                    move |key: String| {
                        vars.lock().unwrap().remove(&key);
                        if has_target {
                            target_vars.lock().unwrap().remove(&key);
                        }
                    }
                }),
            )
            .map_err(|e| format!("Failed to set env.remove: {}", e))?;

        // getAll 返回所有环境变量（JSON 字符串，用户可通过 JSON.parse 解析）
        env_obj
            .set(
                "getAll",
                Function::new(ctx.clone(), {
                    let vars = env_vars.clone();
                    move || {
                        let map = vars.lock().unwrap().clone();
                        serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())
                    }
                }),
            )
            .map_err(|e| format!("Failed to set env.getAll: {}", e))?;

        fm.set("environment", env_obj)
            .map_err(|e| format!("Failed to set environment: {}", e))?;

        // ===== collection API =====
        // get 操作所有集合变量，set/remove 操作目标集合变量
        let coll_obj =
            Object::new(ctx.clone()).map_err(|e| format!("Failed to create coll_obj: {}", e))?;

        coll_obj
            .set(
                "get",
                Function::new(ctx.clone(), {
                    let vars = coll_vars.clone();
                    move |key: String| vars.lock().unwrap().get(&key).cloned()
                }),
            )
            .map_err(|e| format!("Failed to set coll.get: {}", e))?;

        // set 同时更新所有变量和目标变量
        coll_obj
            .set(
                "set",
                Function::new(ctx.clone(), {
                    let vars = coll_vars.clone();
                    let target_vars = target_coll_vars.clone();
                    let has_target = target_collection_id.is_some();
                    move |key: String, value: String| {
                        vars.lock().unwrap().insert(key.clone(), value.clone());
                        if has_target {
                            target_vars.lock().unwrap().insert(key, value);
                        }
                    }
                }),
            )
            .map_err(|e| format!("Failed to set coll.set: {}", e))?;

        // remove 同时从所有变量和目标变量中删除
        coll_obj
            .set(
                "remove",
                Function::new(ctx.clone(), {
                    let vars = coll_vars.clone();
                    let target_vars = target_coll_vars.clone();
                    let has_target = target_collection_id.is_some();
                    move |key: String| {
                        vars.lock().unwrap().remove(&key);
                        if has_target {
                            target_vars.lock().unwrap().remove(&key);
                        }
                    }
                }),
            )
            .map_err(|e| format!("Failed to set coll.remove: {}", e))?;

        // getAll 返回所有集合变量（JSON 字符串，用户可通过 JSON.parse 解析）
        coll_obj
            .set(
                "getAll",
                Function::new(ctx.clone(), {
                    let vars = coll_vars.clone();
                    move || {
                        let map = vars.lock().unwrap().clone();
                        serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())
                    }
                }),
            )
            .map_err(|e| format!("Failed to set coll.getAll: {}", e))?;

        fm.set("collection", coll_obj)
            .map_err(|e| format!("Failed to set collection: {}", e))?;

        // ===== request API =====
        let req_obj =
            Object::new(ctx.clone()).map_err(|e| format!("Failed to create req_obj: {}", e))?;

        req_obj
            .set(
                "getUrl",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move || req.lock().unwrap().url.clone()
                }),
            )
            .map_err(|e| format!("Failed to set getUrl: {}", e))?;

        req_obj
            .set(
                "setUrl",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move |url: String| {
                        req.lock().unwrap().url = url;
                    }
                }),
            )
            .map_err(|e| format!("Failed to set setUrl: {}", e))?;

        req_obj
            .set(
                "getBaseUrl",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move || Self::extract_base_url(&req.lock().unwrap().url)
                }),
            )
            .map_err(|e| format!("Failed to set getBaseUrl: {}", e))?;

        req_obj
            .set(
                "setBaseUrl",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move |base_url: String| {
                        let mut r = req.lock().unwrap();
                        let path = Self::extract_path(&r.url);
                        r.url = Self::build_url(&base_url, &path);
                    }
                }),
            )
            .map_err(|e| format!("Failed to set setBaseUrl: {}", e))?;

        req_obj
            .set(
                "getPath",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move || Self::extract_path(&req.lock().unwrap().url)
                }),
            )
            .map_err(|e| format!("Failed to set getPath: {}", e))?;

        req_obj
            .set(
                "setPath",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move |path: String| {
                        let mut r = req.lock().unwrap();
                        let base = Self::extract_base_url(&r.url);
                        r.url = Self::build_url(&base, &path);
                    }
                }),
            )
            .map_err(|e| format!("Failed to set setPath: {}", e))?;

        req_obj
            .set(
                "getMethod",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move || req.lock().unwrap().method.clone()
                }),
            )
            .map_err(|e| format!("Failed to set getMethod: {}", e))?;

        req_obj
            .set(
                "setMethod",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move |method: String| {
                        req.lock().unwrap().method = method;
                    }
                }),
            )
            .map_err(|e| format!("Failed to set setMethod: {}", e))?;

        req_obj
            .set(
                "getHeader",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move |key: String| {
                        let lower_key = key.to_lowercase();
                        req.lock()
                            .unwrap()
                            .headers
                            .iter()
                            .find(|h| h.key.to_lowercase() == lower_key)
                            .map(|h| h.value.clone())
                    }
                }),
            )
            .map_err(|e| format!("Failed to set getHeader: {}", e))?;

        req_obj
            .set(
                "setHeader",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move |key: String, value: String| {
                        let lower_key = key.to_lowercase();
                        let mut r = req.lock().unwrap();
                        let existing = r
                            .headers
                            .iter_mut()
                            .find(|h| h.key.to_lowercase() == lower_key);
                        if let Some(h) = existing {
                            h.value = value;
                        } else {
                            r.headers.push(Header {
                                key,
                                value,
                                enabled: true,
                                description: None,
                            });
                        }
                    }
                }),
            )
            .map_err(|e| format!("Failed to set setHeader: {}", e))?;

        req_obj
            .set(
                "removeHeader",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move |key: String| {
                        let lower_key = key.to_lowercase();
                        req.lock()
                            .unwrap()
                            .headers
                            .retain(|h| h.key.to_lowercase() != lower_key);
                    }
                }),
            )
            .map_err(|e| format!("Failed to set removeHeader: {}", e))?;

        // getHeaders 返回所有启用的请求头（JSON 字符串，用户可通过 JSON.parse 解析）
        req_obj
            .set(
                "getHeaders",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move || {
                        let headers: Vec<serde_json::Value> = req
                            .lock()
                            .unwrap()
                            .headers
                            .iter()
                            .filter(|h| h.enabled)
                            .map(|h| {
                                serde_json::json!({
                                    "key": h.key.clone(),
                                    "value": h.value.clone()
                                })
                            })
                            .collect();
                        serde_json::to_string(&headers).unwrap_or_else(|_| "[]".to_string())
                    }
                }),
            )
            .map_err(|e| format!("Failed to set getHeaders: {}", e))?;

        req_obj
            .set(
                "getBody",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move || req.lock().unwrap().body.clone()
                }),
            )
            .map_err(|e| format!("Failed to set getBody: {}", e))?;

        req_obj
            .set(
                "setBody",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move |body: String| {
                        req.lock().unwrap().body = Some(body);
                    }
                }),
            )
            .map_err(|e| format!("Failed to set setBody: {}", e))?;

        // ===== params API =====
        req_obj
            .set(
                "getParam",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move |key: String| {
                        let lower_key = key.to_lowercase();
                        req.lock()
                            .unwrap()
                            .params
                            .iter()
                            .find(|p| p.key.to_lowercase() == lower_key)
                            .map(|p| p.value.clone())
                    }
                }),
            )
            .map_err(|e| format!("Failed to set getParam: {}", e))?;

        req_obj
            .set(
                "setParam",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move |key: String, value: String| {
                        let lower_key = key.to_lowercase();
                        let mut r = req.lock().unwrap();
                        let existing = r
                            .params
                            .iter_mut()
                            .find(|p| p.key.to_lowercase() == lower_key);
                        if let Some(p) = existing {
                            p.value = value;
                        } else {
                            r.params.push(Param {
                                key,
                                value,
                                enabled: true,
                                description: None,
                            });
                        }
                    }
                }),
            )
            .map_err(|e| format!("Failed to set setParam: {}", e))?;

        req_obj
            .set(
                "removeParam",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move |key: String| {
                        let lower_key = key.to_lowercase();
                        req.lock()
                            .unwrap()
                            .params
                            .retain(|p| p.key.to_lowercase() != lower_key);
                    }
                }),
            )
            .map_err(|e| format!("Failed to set removeParam: {}", e))?;

        req_obj
            .set(
                "getParams",
                Function::new(ctx.clone(), {
                    let req = request_state.clone();
                    move || {
                        // 返回 JSON 字符串，用户可通过 JSON.parse 解析
                        let params: Vec<serde_json::Value> = req
                            .lock()
                            .unwrap()
                            .params
                            .iter()
                            .filter(|p| p.enabled)
                            .map(|p| {
                                serde_json::json!({
                                    "key": p.key.clone(),
                                    "value": p.value.clone()
                                })
                            })
                            .collect();
                        serde_json::to_string(&params).unwrap_or_else(|_| "[]".to_string())
                    }
                }),
            )
            .map_err(|e| format!("Failed to set getParams: {}", e))?;

        fm.set("request", req_obj)
            .map_err(|e| format!("Failed to set request: {}", e))?;

        // ===== response API (仅后置脚本) =====
        if let Some(resp) = response {
            let resp_obj = Object::new(ctx.clone())
                .map_err(|e| format!("Failed to create resp_obj: {}", e))?;

            let status = resp.status;
            resp_obj
                .set("getStatus", Function::new(ctx.clone(), move || status))
                .map_err(|e| format!("Failed to set getStatus: {}", e))?;

            let status_text = resp.status_text.clone();
            resp_obj
                .set(
                    "getStatusText",
                    Function::new(ctx.clone(), move || status_text.clone()),
                )
                .map_err(|e| format!("Failed to set getStatusText: {}", e))?;

            let resp_headers = resp.headers.clone();
            resp_obj
                .set(
                    "getHeader",
                    Function::new(ctx.clone(), {
                        let headers = resp_headers.clone();
                        move |key: String| {
                            let lower_key = key.to_lowercase();
                            headers
                                .get(&lower_key)
                                .or_else(|| headers.get(&key))
                                .cloned()
                        }
                    }),
                )
                .map_err(|e| format!("Failed to set response.getHeader: {}", e))?;

            // getHeaders 返回所有响应头（JSON 字符串，用户可通过 JSON.parse 解析）
            resp_obj
                .set(
                    "getHeaders",
                    Function::new(ctx.clone(), {
                        let headers = resp_headers.clone();
                        move || serde_json::to_string(&headers).unwrap_or_else(|_| "{}".to_string())
                    }),
                )
                .map_err(|e| format!("Failed to set response.getHeaders: {}", e))?;

            let time = resp.time;
            resp_obj
                .set("getTime", Function::new(ctx.clone(), move || time))
                .map_err(|e| format!("Failed to set getTime: {}", e))?;

            let size = resp.size;
            resp_obj
                .set("getSize", Function::new(ctx.clone(), move || size))
                .map_err(|e| format!("Failed to set getSize: {}", e))?;

            let resp_body = resp.body.clone();
            resp_obj
                .set(
                    "getBody",
                    Function::new(ctx.clone(), {
                        let body = resp_body.clone();
                        move || body.clone()
                    }),
                )
                .map_err(|e| format!("Failed to set getBody: {}", e))?;

            // 设置原始响应体供 getJson 辅助函数使用
            resp_obj
                .set("_rawBody", resp_body.clone())
                .map_err(|e| format!("Failed to set _rawBody: {}", e))?;

            fm.set("response", resp_obj)
                .map_err(|e| format!("Failed to set response: {}", e))?;
        }

        // ===== log API =====
        // 支持 null/undefined，自动转换为字符串
        fm.set(
            "log",
            Function::new(ctx.clone(), {
                let logs_ref = logs.clone();
                let src = source.clone();
                move |args: Option<String>| {
                    let message = match args {
                        Some(s) => s,
                        None => "null".to_string(),
                    };
                    logs_ref.lock().unwrap().push(ScriptLog {
                        level: "log".to_string(),
                        message,
                        source: src.clone(),
                    });
                }
            }),
        )
        .map_err(|e| format!("Failed to set log: {}", e))?;

        // ===== sleep API =====
        // 真实延时（限制最大 5 分钟，避免无限阻塞 blocking 线程池）
        fm.set(
            "sleep",
            Function::new(ctx.clone(), |ms: i32| {
                if ms > 0 {
                    let capped = (ms as u64).min(300_000);
                    std::thread::sleep(Duration::from_millis(capped));
                }
            }),
        )
        .map_err(|e| format!("Failed to set sleep: {}", e))?;

        // ===== assert API =====
        if has_response {
            fm.set(
                "assert",
                Function::new(ctx.clone(), {
                    let results = test_results.clone();
                    move |condition: bool, message: Option<String>| {
                        let msg = message.unwrap_or_else(|| "断言检查".to_string());
                        results.lock().unwrap().push(ScriptTestResult {
                            name: msg.clone(),
                            passed: condition,
                            error: if condition { None } else { Some(msg) },
                        });
                    }
                }),
            )
            .map_err(|e| format!("Failed to set assert: {}", e))?;
        } else {
            fm.set(
                "assert",
                Function::new(ctx.clone(), |_condition: bool, _message: Option<String>| {}),
            )
            .map_err(|e| format!("Failed to set assert: {}", e))?;
        }

        // 设置全局 fm 对象
        globals
            .set("fm", fm)
            .map_err(|e| format!("Failed to set global fm: {}", e))?;

        // 执行脚本，添加 getJson 辅助函数
        let wrapped_code = if has_response {
            // 后置脚本：添加 getJson 辅助函数
            format!(
                "(function() {{ \
                    fm.response.getJson = function() {{ \
                        try {{ return JSON.parse(fm.response._rawBody); }} \
                        catch(e) {{ return null; }} \
                    }}; \
                    {}; \
                }})()",
                code
            )
        } else {
            // 前置脚本：直接执行
            format!("(function() {{ {} }})()", code)
        };

        if let Err(e) = ctx.eval::<(), _>(wrapped_code.as_bytes()) {
            // Error::Exception 是单元变体，本身不携带异常信息，
            // 必须通过 CaughtError::from_error(&ctx, e) 取回真实的 JS 异常值（消息 + 堆栈），
            // 否则只会得到无信息的 "Exception generated by QuickJS"。
            let caught = CaughtError::from_error(&ctx, e);
            let detail = match &caught {
                CaughtError::Exception(ex) => {
                    let msg = ex
                        .message()
                        .unwrap_or_else(|| "(无消息的 Error 对象)".to_string());
                    match ex.stack() {
                        Some(stack) if !stack.is_empty() => format!("{}\n{}", msg, stack),
                        _ => msg,
                    }
                }
                CaughtError::Value(v) => {
                    format!("抛出了非 Error 类型的值: {:?}", v)
                }
                CaughtError::Error(other) => format!("{}", other),
            };
            return Err(format!("Script error in {}: {}", source, detail));
        }

        Ok(())
    }

    fn extract_base_url(url: &str) -> String {
        if url.is_empty() {
            return "".to_string();
        }

        if let Some(matched) = url.match_indices("://").next() {
            let start = matched.0;
            if let Some(slash_pos) = url[start + 3..].find('/') {
                return url[..start + 3 + slash_pos].to_string();
            }
            return url.to_string();
        }

        if url.starts_with('/') {
            return "".to_string();
        }

        if let Some(i) = url.find('/') {
            if i > 0 && url[i + 1..].starts_with('/') {
                if let Some(slash_pos) = url[i + 2..].find('/') {
                    return url[..i + 2 + slash_pos].to_string();
                }
                return url.to_string();
            } else if i > 0 {
                return url[..i].to_string();
            }
        }

        url.to_string()
    }

    fn extract_path(url: &str) -> String {
        if url.is_empty() {
            return "".to_string();
        }
        let base = Self::extract_base_url(url);
        if base.is_empty() {
            return url.to_string();
        }
        let path = url[base.len()..].to_string();
        if path.is_empty() {
            "/".to_string()
        } else {
            path
        }
    }

    fn build_url(base: &str, path: &str) -> String {
        if base.is_empty() {
            return path.to_string();
        }
        if path.is_empty() {
            return base.to_string();
        }
        let path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };
        format!("{}{}", base, path)
    }
}
