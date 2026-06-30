//! WebSocket 命令接口
//!
//! 提供 WebSocket 相关的 Tauri 命令，调用应用服务进行业务处理。
//!
//! 命令层是 thin wrapper，所有持久化操作通过 application service 走 Repository，
//! 不再在命令里直接执行 SQL（保持 DDD 分层）。

use crate::application::services::WsApplicationService;
use crate::domain::models::{WsConfigEntry, WsHeader, WsParam};
use tauri::AppHandle;

/// 建立 WebSocket 连接
#[tauri::command]
pub async fn connect_websocket(
    app: AppHandle,
    url: String,
    headers: Vec<WsHeader>,
    params: Vec<WsParam>,
    workspace_id: String,
    ws_id: Option<String>,
) -> Result<(), String> {
    WsApplicationService::connect_ws(app, url, headers, params, workspace_id, ws_id).await
}

/// 发送 WebSocket 消息
#[tauri::command]
pub async fn send_ws_message(
    app: AppHandle,
    content: String,
    message_type: String,
) -> Result<(), String> {
    WsApplicationService::send_ws_message(app, content, message_type).await
}

/// 断开 WebSocket 连接
#[tauri::command]
pub async fn disconnect_websocket(app: AppHandle) -> Result<(), String> {
    WsApplicationService::disconnect_ws(app).await
}

/// 检查 WebSocket 连接状态
#[tauri::command]
pub async fn is_ws_connected() -> Result<bool, String> {
    Ok(WsApplicationService::is_connected().await)
}

/// 获取所有 WebSocket 配置
#[tauri::command]
pub fn get_ws_configs(workspace_id: String) -> Result<Vec<WsConfigEntry>, String> {
    WsApplicationService::list_ws_configs(&workspace_id)
}

/// 保存 WebSocket 配置（新建或更新）
#[tauri::command]
pub fn save_ws_config(
    workspace_id: String,
    id: Option<String>,
    name: String,
    url: String,
    headers: Vec<WsHeader>,
    params: Vec<WsParam>,
) -> Result<String, String> {
    WsApplicationService::save_ws_config(&workspace_id, id, name, url, headers, params)
}

/// 删除 WebSocket 配置
#[tauri::command]
pub fn delete_ws_config(workspace_id: String, id: String) -> Result<(), String> {
    WsApplicationService::delete_ws_config(&workspace_id, &id)
}
