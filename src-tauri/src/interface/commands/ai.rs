//! AI 命令接口
//!
//! 提供 AI 相关的 Tauri 命令，处理前端交互。

use crate::application::services::AiApplicationService;
use crate::domain::models::AiChatMessage;
use tauri::AppHandle;

/// 获取 AI 模型列表
#[tauri::command]
pub async fn get_ai_models(
    api_endpoint: String,
    api_key: Option<String>,
    custom_headers: Option<Vec<crate::domain::models::Header>>,
) -> Result<Vec<String>, String> {
    AiApplicationService::get_ai_models(api_endpoint, api_key, custom_headers).await
}

/// AI 聊天（流式）
#[tauri::command]
pub async fn chat_ai(app: AppHandle, messages: Vec<AiChatMessage>) -> Result<String, String> {
    AiApplicationService::chat_ai(app, messages).await
}

/// AI 聊天（@fm 工作区上下文，Function Calling Agent 模式）
#[tauri::command]
pub async fn chat_ai_agent(
    app: AppHandle,
    workspace_id: String,
    messages: Vec<AiChatMessage>,
) -> Result<String, String> {
    AiApplicationService::chat_ai_agent(app, workspace_id, messages).await
}

/// AI 优化脚本
#[tauri::command]
pub async fn optimize_script_ai(
    app: AppHandle,
    script_content: String,
    script_type: String,
) -> Result<String, String> {
    AiApplicationService::optimize_script_ai(app, script_content, script_type).await
}
