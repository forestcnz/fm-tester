use serde::{Deserialize, Serialize};

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    /// 思考过程内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// 聊天会话（完整数据，存储在单独文件中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub created_at: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// 聊天会话索引条目（存储在 index.toml 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionIndex {
    pub id: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub message_count: usize,
}

/// 聊天索引文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatIndex {
    pub sessions: Vec<ChatSessionIndex>,
    pub active_session_id: Option<String>,
}
