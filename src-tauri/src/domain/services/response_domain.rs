//! Response 领域服务
//!
//! 提供响应快照相关的纯业务逻辑（ID生成、实体创建、MD文档生成）。

use crate::domain::models::common::generate_id;
use crate::domain::models::{SavedResponse, SavedResponseIndexEntry};
use chrono::Local;

/// Response 领域服务
pub struct ResponseDomainService;

impl ResponseDomainService {
    /// 生成新的响应ID
    pub fn generate_id() -> String {
        generate_id("resp")
    }

    /// 生成当前时间戳（本地时间）
    pub fn generate_timestamp() -> String {
        Local::now().to_rfc3339()
    }

    /// 创建 SavedResponse 实体
    pub fn create_saved_response(
        name: String,
        api_id: Option<String>,
        doc_content: String,
    ) -> SavedResponse {
        let id = Self::generate_id();
        let created_at = Self::generate_timestamp();

        SavedResponse {
            id,
            name,
            created_at,
            api_id,
            doc_content,
        }
    }

    /// 创建索引条目
    pub fn create_index_entry(saved_response: &SavedResponse) -> SavedResponseIndexEntry {
        SavedResponseIndexEntry {
            id: saved_response.id.clone(),
            name: saved_response.name.clone(),
            created_at: saved_response.created_at.clone(),
            api_id: saved_response.api_id.clone(),
        }
    }
}
