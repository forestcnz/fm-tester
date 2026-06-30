use crate::application::services::ChatApplicationService;
use crate::domain::models::{ChatMessage, ChatSession};
use tauri::AppHandle;

/// 保存聊天记录
#[tauri::command]
pub fn save_chat_history(
    app: AppHandle,
    workspace_id: String,
    session_id: Option<String>,
    messages: Vec<ChatMessage>,
) -> Result<String, String> {
    let service = ChatApplicationService;
    service.save_chat_history(&app, &workspace_id, session_id, messages)
}

/// 获取聊天记录
#[tauri::command]
pub fn get_chat_history(
    workspace_id: String,
    session_id: Option<String>,
) -> Result<Vec<ChatMessage>, String> {
    let service = ChatApplicationService;
    service.get_chat_history(&workspace_id, session_id)
}

/// 清空聊天记录
#[tauri::command]
pub fn clear_chat_history(workspace_id: String, session_id: Option<String>) -> Result<(), String> {
    let service = ChatApplicationService;
    service.clear_chat_history(&workspace_id, session_id)
}

/// 获取聊天会话列表（仅索引信息）
#[tauri::command]
pub fn get_chat_sessions(workspace_id: String) -> Result<Vec<ChatSession>, String> {
    let service = ChatApplicationService;
    service.get_chat_sessions(&workspace_id)
}

/// 删除聊天会话
#[tauri::command]
pub fn delete_chat_session(workspace_id: String, session_id: String) -> Result<(), String> {
    let service = ChatApplicationService;
    service.delete_chat_session(&workspace_id, &session_id)
}

/// 重命名聊天会话
#[tauri::command]
pub fn rename_chat_session(
    workspace_id: String,
    session_id: String,
    new_title: String,
) -> Result<(), String> {
    let service = ChatApplicationService;
    service.rename_chat_session(&workspace_id, &session_id, &new_title)
}
