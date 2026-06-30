use super::Header;
use serde::{Deserialize, Serialize};

/// SSE 事件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseEvent {
    /// 事件 ID（可选，用于重连）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// 事件类型（默认为 "message"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    /// 事件数据
    pub data: String,
    /// 接收时间戳（毫秒）
    pub timestamp: u64,
}

/// SSE 配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseConfig {
    /// SSE 端点 URL
    pub url: String,
    /// 自定义请求头
    pub headers: Vec<Header>,
    /// 超时时间（毫秒）
    pub timeout_ms: u64,
    /// 最后接收的事件 ID（用于重连）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_event_id: Option<String>,
}

/// SSE 连接状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SseStatus {
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 已断开
    Disconnected,
    /// 出错
    Error,
}

/// SSE 响应状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseState {
    /// 连接状态
    pub status: SseStatus,
    /// 接收的事件列表
    pub events: Vec<SseEvent>,
    /// 错误信息（如果出错）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 连接时间戳
    pub connected_at: u64,
}
