//! WebSocket 数据模型
//!
//! 定义 WebSocket 连接、消息、状态等核心数据结构。

use serde::{Deserialize, Serialize};

/// WebSocket 连接状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WsStatus {
    /// 正在连接
    Connecting,
    /// 已连接
    Connected,
    /// 已断开
    Disconnected,
    /// 连接错误
    Error,
}

/// WebSocket 消息
///
/// 表示一条 WebSocket 消息，包含方向、内容、类型等信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    /// 消息唯一标识
    pub id: String,
    /// 消息方向："sent" 或 "received"
    pub direction: String,
    /// 消息内容（文本或 Base64 编码的二进制数据）
    pub content: String,
    /// 消息类型："text" 或 "binary"
    #[serde(rename = "type")]
    pub message_type: String,
    /// 消息时间戳（毫秒）
    pub timestamp: u64,
}

/// WebSocket 状态事件
///
/// 用于向前端推送连接状态变化。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsState {
    /// 当前连接状态
    pub status: WsStatus,
    /// 错误信息（如果有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 连接建立时间（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected_at: Option<u64>,
}

/// WebSocket 连接配置
///
/// 用于存储 WebSocket 连接的配置参数。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WsConfig {
    /// WebSocket URL（ws:// 或 wss://）
    pub url: String,
    /// 连接参数（转换为 URL query string）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<WsParam>>,
    /// 请求头
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<WsHeader>>,
    /// 心跳间隔（毫秒），可选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_interval: Option<u64>,
    /// 心跳消息内容，可选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_message: Option<String>,
    /// 重连尝试次数，可选（0 表示不自动重连）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reconnect_attempts: Option<u32>,
    /// 重连延迟（毫秒），可选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reconnect_delay: Option<u64>,
}

/// WebSocket 配置项（侧边栏的 ws_configs 表对应实体）
///
/// 与 `WsConfig`（用于实际连接的运行时参数）不同，本结构用于
/// 持久化用户在侧边栏新建的 WebSocket 配置项（含 id、name、时间戳）。
/// 序列化为驼峰以保持与前端契约一致。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsConfigEntry {
    /// 配置 ID
    pub id: String,
    /// 显示名称
    pub name: String,
    /// WebSocket URL
    pub url: String,
    /// 请求头（默认空数组）
    #[serde(default)]
    pub headers: Vec<WsHeader>,
    /// URL 参数（默认空数组）
    #[serde(default)]
    pub params: Vec<WsParam>,
    /// 创建时间（RFC3339）
    pub created_at: String,
    /// 更新时间（RFC3339）
    pub updated_at: String,
}

/// WebSocket 参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsParam {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// WebSocket 请求头
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsHeader {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl WsConfig {
    /// 验证 WebSocket 配置
    pub fn validate(&self) -> Result<(), String> {
        if self.url.trim().is_empty() {
            return Err("WebSocket URL 不能为空".to_string());
        }

        // 检查 URL 协议
        let url_lower = self.url.to_lowercase();
        if !url_lower.starts_with("ws://") && !url_lower.starts_with("wss://") {
            return Err("WebSocket URL 必须以 ws:// 或 wss:// 开头".to_string());
        }

        // 验证心跳配置
        if let Some(interval) = self.heartbeat_interval {
            if interval < 1000 {
                return Err("心跳间隔不能小于 1000 毫秒".to_string());
            }
            if self.heartbeat_message.is_none() {
                return Err("启用心跳时必须指定心跳消息内容".to_string());
            }
        }

        // 验证重连配置
        if let Some(attempts) = self.reconnect_attempts {
            if attempts > 10 {
                return Err("重连次数不能超过 10 次".to_string());
            }
            if self.reconnect_delay.is_none() {
                return Err("启用重连时必须指定重连延迟".to_string());
            }
        }

        Ok(())
    }
}

impl WsMessage {
    /// 创建发送的消息
    pub fn new_sent(content: String, message_type: String) -> Self {
        use crate::domain::models::common::generate_id;
        use std::time::{SystemTime, UNIX_EPOCH};

        Self {
            id: generate_id("msg"),
            direction: "sent".to_string(),
            content,
            message_type,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    /// 创建接收的消息
    pub fn new_received(content: String, message_type: String) -> Self {
        use crate::domain::models::common::generate_id;
        use std::time::{SystemTime, UNIX_EPOCH};

        Self {
            id: generate_id("msg"),
            direction: "received".to_string(),
            content,
            message_type,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}
