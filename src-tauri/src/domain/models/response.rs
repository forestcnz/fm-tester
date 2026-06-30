use serde::{Deserialize, Serialize};

/// 保存的响应（简化版，只保留基本信息和 MD 文档）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedResponse {
    pub id: String,
    pub name: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_id: Option<String>,
    pub doc_content: String,
}

/// 响应索引条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedResponseIndexEntry {
    pub id: String,
    pub name: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_id: Option<String>,
}

/// 响应索引文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavedResponsesIndex {
    pub responses: Vec<SavedResponseIndexEntry>,
}
