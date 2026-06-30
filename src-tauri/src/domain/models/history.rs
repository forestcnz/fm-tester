use super::{FormField, Header};
use serde::{Deserialize, Serialize};

/// 历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub method: String,
    pub url: String,
    pub resolved_url: String,
    pub headers: Vec<Header>,
    pub body: Option<String>,
    pub body_type: Option<String>,
    pub form_fields: Option<Vec<FormField>>,
    pub status: u16,
    pub status_text: String,
    pub response_headers: std::collections::HashMap<String, String>,
    pub response_body: String,
    pub time: u64,
    pub size: u64,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_name: Option<String>,
}

/// 历史记录存储结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoryConfig {
    pub entries: Vec<HistoryEntry>,
}
