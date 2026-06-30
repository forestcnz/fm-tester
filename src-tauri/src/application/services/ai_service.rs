//! AI 应用服务
//!
//! 处理 AI 相关的 UI 交互，协调基础设施 HTTP 客户端和领域服务。
//! 任务状态管理（运行时状态）放在 Application 层。

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::AiToolService;
use crate::domain::models::AiChatMessage;
use crate::domain::repositories::AiHttpClientService;
use crate::domain::services::{AiDomainService, EncryptionService};
use crate::infrastructure::{get_ai_http_client, get_encryption_service, RepositoryFactory};
use tauri::{AppHandle, Emitter};

/// 生成任务状态（用于取消检查）
#[derive(Debug, Clone)]
pub struct GenerationTaskState {
    pub cancelled: bool,
    pub start_time: Instant,
}

lazy_static! {
    /// 任务状态管理（运行时状态，放在 Application 层）
    static ref GENERATION_TASK_STATE: Arc<Mutex<HashMap<String, GenerationTaskState>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// AI 应用服务
pub struct AiApplicationService;

impl AiApplicationService {
    /// 创建默认实例
    pub fn new() -> Self {
        Self
    }

    /// 初始化生成任务状态
    pub fn init_generation_task(api_id: &str) {
        let mut state = GENERATION_TASK_STATE.lock().unwrap();
        state.insert(
            api_id.to_string(),
            GenerationTaskState {
                cancelled: false,
                start_time: Instant::now(),
            },
        );
    }

    /// 检查生成任务是否被取消
    pub fn is_generation_cancelled(api_id: &str) -> bool {
        let state = GENERATION_TASK_STATE.lock().unwrap();
        if let Some(task) = state.get(api_id) {
            task.cancelled
        } else {
            false
        }
    }

    /// 取消生成任务
    pub fn cancel_generation_task(api_id: &str) {
        let mut state = GENERATION_TASK_STATE.lock().unwrap();
        if let Some(task) = state.get_mut(api_id) {
            task.cancelled = true;
        }
    }

    /// 清理生成任务状态
    pub fn cleanup_generation_task(api_id: &str) {
        let mut state = GENERATION_TASK_STATE.lock().unwrap();
        state.remove(api_id);
    }

    /// 获取生成任务耗时
    pub fn get_generation_elapsed_seconds(api_id: &str) -> u64 {
        let state = GENERATION_TASK_STATE.lock().unwrap();
        if let Some(task) = state.get(api_id) {
            task.start_time.elapsed().as_secs()
        } else {
            0
        }
    }

    /// 检查生成任务是否存在且未取消
    pub fn is_generation_running(api_id: &str) -> bool {
        let state = GENERATION_TASK_STATE.lock().unwrap();
        if let Some(task) = state.get(api_id) {
            !task.cancelled
        } else {
            false
        }
    }

    /// 获取 AI 模型列表
    pub async fn get_ai_models(
        api_endpoint: String,
        api_key: Option<String>,
        custom_headers: Option<Vec<crate::domain::models::Header>>,
    ) -> Result<Vec<String>, String> {
        if api_endpoint.is_empty() {
            return Err("请先配置 AI API Endpoint".to_string());
        }

        // 复用 load_ai_settings 统一处理 endpoint/key/headers 校验与解密
        // model 此处用不到，传占位值跳过 model 非空校验（获取列表本就用于选择 model）
        let (api_endpoint, decrypted_key, _model, headers, _timeout) = Self::load_ai_settings(
            Some(api_endpoint),
            api_key,
            Some("(pending)".to_string()),
            custom_headers,
            None,
        )
        .await?;

        let client = get_ai_http_client();
        client
            .get_models(&api_endpoint, &decrypted_key, headers)
            .await
    }

    /// 加载 AI 调用所需配置：endpoint / api_key / model / headers / timeout
    ///
    /// 抽出公共逻辑，避免在 get_ai_models / chat_ai / optimize_script_ai 中重复样板。
    /// 各方法可通过参数覆盖配置中的字段（如前端传入了新 endpoint / api_key 等）。
    async fn load_ai_settings(
        endpoint_override: Option<String>,
        api_key_override: Option<String>,
        model_override: Option<String>,
        headers_override: Option<Vec<crate::domain::models::Header>>,
        timeout_override: Option<u64>,
    ) -> Result<
        (
            String,
            String,
            String,
            Option<Vec<crate::domain::models::Header>>,
            u64,
        ),
        String,
    > {
        let settings_repo = RepositoryFactory::get_app_config_repository();
        let settings = settings_repo.read()?;
        let cfg = &settings.settings.ai;

        let api_endpoint = endpoint_override.unwrap_or(cfg.api_endpoint.clone());
        if api_endpoint.is_empty() {
            return Err("请先配置 AI API Endpoint".to_string());
        }

        // 解密 key
        let encryption_service = get_encryption_service();
        let api_key = match api_key_override {
            Some(k) if !k.is_empty() => k,
            _ => {
                if cfg.encrypted_api_key.is_empty() {
                    return Err("请先配置 AI API Key".to_string());
                }
                encryption_service
                    .decrypt(&cfg.encrypted_api_key)
                    .map_err(|e| format!("解密 API Key 失败: {}", e))?
            }
        };

        let model = model_override.unwrap_or(cfg.model.clone());
        if model.is_empty() {
            return Err("请先配置 AI Model".to_string());
        }

        let headers = match headers_override {
            Some(h) => Some(h),
            None => {
                if cfg.custom_headers.is_empty() {
                    None
                } else {
                    Some(cfg.custom_headers.clone())
                }
            }
        };

        let timeout = timeout_override.unwrap_or(cfg.timeout);

        Ok((api_endpoint, api_key, model, headers, timeout))
    }

    /// AI 聊天（流式）
    pub async fn chat_ai(app: AppHandle, messages: Vec<AiChatMessage>) -> Result<String, String> {
        let (api_endpoint, api_key, model, custom_headers, timeout) =
            Self::load_ai_settings(None, None, None, None, None).await?;

        let client = get_ai_http_client();
        client
            .chat_stream(
                app,
                &api_endpoint,
                &api_key,
                &model,
                messages,
                custom_headers,
                timeout,
                None,
            )
            .await
    }

    /// AI 聊天（@fm 工作区上下文，Function Calling Agent 模式）
    ///
    /// 注入工作区工具定义，AI 可多轮调用工具查询接口信息后作答。
    /// content/reasoning 通过事件流式推送；工具执行通过 `ai-chat-tool` 事件通知前端。
    pub async fn chat_ai_agent(
        app: AppHandle,
        workspace_id: String,
        messages: Vec<AiChatMessage>,
    ) -> Result<String, String> {
        let (api_endpoint, api_key, model, custom_headers, timeout) =
            Self::load_ai_settings(None, None, None, None, None).await?;

        // 注入 system prompt：替换已有首条 system，否则插入到开头
        let mut history = messages;
        let sys_msg = AiChatMessage::system(AiDomainService::get_workspace_chat_system_prompt());
        if !history.is_empty() && history[0].role == "system" {
            history[0] = sys_msg;
        } else {
            history.insert(0, sys_msg);
        }

        let tools = AiDomainService::get_workspace_tools();
        let client = get_ai_http_client();
        const MAX_ITERATIONS: usize = 8;

        for _ in 0..MAX_ITERATIONS {
            let result = client
                .chat_with_tools(
                    app.clone(),
                    &api_endpoint,
                    &api_key,
                    &model,
                    history.clone(),
                    tools.clone(),
                    custom_headers.clone(),
                    timeout,
                    None,
                )
                .await?;

            // 无工具调用：纯文本答案，已通过事件流式输出
            if !result.has_tool_calls() {
                return Ok(result.content);
            }

            // 追加 assistant 消息（携带 tool_calls）
            history.push(AiChatMessage::assistant(
                if result.content.is_empty() {
                    None
                } else {
                    Some(result.content.clone())
                },
                Some(result.tool_calls.clone()),
            ));

            // 逐个执行工具并追加 tool 结果消息
            for tc in &result.tool_calls {
                app.emit(
                    "ai-chat-tool",
                    serde_json::json!({
                        "name": tc.function.name,
                        "arguments": tc.function.arguments,
                    }),
                )
                .ok();
                let tool_result = AiToolService::execute(
                    &workspace_id,
                    &tc.function.name,
                    &tc.function.arguments,
                );
                history.push(AiChatMessage::tool(&tc.id, tool_result));
            }
            // 继续下一轮，让 AI 基于工具结果继续生成
        }

        Ok("（工具调用轮次已达上限，请缩小问题范围后重试）".to_string())
    }

    /// AI 优化脚本
    pub async fn optimize_script_ai(
        app: AppHandle,
        script_content: String,
        script_type: String,
    ) -> Result<String, String> {
        let (api_endpoint, api_key, model, custom_headers, timeout) =
            Self::load_ai_settings(None, None, None, None, None).await?;

        let system_prompt = AiDomainService::get_script_system_prompt(&script_type);

        let user_content = if script_content.trim().is_empty() {
            "请帮我生成一个基础的脚本模板。".to_string()
        } else {
            format!("请优化或完善以下脚本：\n\n{}", script_content)
        };
        let messages = vec![
            AiChatMessage::system(system_prompt),
            AiChatMessage::user(user_content),
        ];

        let client = get_ai_http_client();
        client
            .chat_stream(
                app,
                &api_endpoint,
                &api_key,
                &model,
                messages,
                custom_headers,
                timeout,
                None,
            )
            .await
    }

    /// 根据聊天消息生成会话标题（非流式）
    ///
    /// 取对话前若干条消息，要求模型输出简短中文标题。
    /// 用于会话列表的自动总结标题，失败时返回 Err（由调用方回退到临时标题）。
    pub async fn generate_chat_title(messages: Vec<AiChatMessage>) -> Result<String, String> {
        let (api_endpoint, api_key, model, custom_headers, timeout) =
            Self::load_ai_settings(None, None, None, None, None).await?;

        let mut title_messages = vec![AiChatMessage::system(
            AiDomainService::get_chat_summary_prompt(),
        )];
        title_messages.extend(messages);

        let client = get_ai_http_client();
        let raw = client
            .chat(
                &api_endpoint,
                &api_key,
                &model,
                title_messages,
                custom_headers,
                timeout,
            )
            .await?;

        // 清理模型可能附带的引号、标点与换行
        let raw_trimmed = raw.trim();
        let stripped = raw_trimmed.trim_matches(|c| {
            matches!(
                c,
                '"' | '\'' | '「' | '」' | '\u{201C}' | '\u{201D}' | '《' | '》' | '.' | '。'
            )
        });
        let cleaned = stripped
            .lines()
            .next()
            .unwrap_or(stripped)
            .trim()
            .to_string();

        if cleaned.is_empty() {
            Err("生成的标题为空".to_string())
        } else {
            Ok(cleaned)
        }
    }
}

impl Default for AiApplicationService {
    fn default() -> Self {
        Self::new()
    }
}
