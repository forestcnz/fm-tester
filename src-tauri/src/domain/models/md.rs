use serde::{Deserialize, Serialize};

/// 文档索引条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocIndexEntry {
    pub api_id: String,
    /// 最新编辑保存时间
    pub updated_at: String,
}

/// 文档索引文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocIndex {
    pub entries: Vec<DocIndexEntry>,
}

/// 文档生成状态
#[derive(Debug, Clone, Serialize)]
pub struct DocGenerationStatus {
    pub api_id: String,
    pub generating: bool,
    pub elapsed_seconds: u64,
    pub error: Option<String>,
}

/// 文档元数据
#[derive(Debug, Clone, Serialize)]
pub struct DocMetadata {
    pub api_id: String,
    pub updated_at: Option<String>,
}
