//! SSE 相关 Tauri 命令
//!
//! 提供 SSE 连接和管理的 Tauri 命令接口。

use crate::domain::models::Header;
use crate::infrastructure::get_sse_client;
use tauri::{command, AppHandle};

/// 开始 SSE 连接
#[command]
pub async fn start_sse_cmd(
    app: AppHandle,
    method: String,
    url: String,
    headers: Vec<Header>,
    body: Option<String>,
    timeout_ms: u64,
    last_event_id: Option<String>,
) -> Result<(), String> {
    // 使用全局单例（与 http_service 检测到 SSE 后转流共用同一实例，
    // 这样 stop_sse_cmd 的取消信号才能正确传递）
    let client = get_sse_client();

    client
        .start_sse(app, &method, &url, headers, body, timeout_ms, last_event_id)
        .await
}

/// 停止 SSE 连接
#[command]
pub fn stop_sse_cmd() {
    get_sse_client().stop_sse();
}
