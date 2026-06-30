//! Response 应用服务
//!
//! 协调响应快照的业务操作，通过仓储工厂动态获取仓储。

use crate::domain::models::{SavedResponse, SavedResponseIndexEntry};
use crate::domain::services::ResponseDomainService;
use crate::infrastructure::RepositoryFactory;

/// Response 应用服务
pub struct ResponseApplicationService;

impl ResponseApplicationService {
    /// 格式化时间为友好格式
    fn format_time(time_str: &str) -> String {
        if time_str.is_empty() {
            return String::new();
        }
        // 解析 RFC3339 格式时间
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(time_str) {
            dt.format("%Y-%m-%d %H:%M").to_string()
        } else {
            time_str.to_string()
        }
    }

    /// 创建保存的响应实体（封装Domain服务）
    pub fn create_saved_response(
        name: String,
        api_id: Option<String>,
        doc_content: String,
    ) -> SavedResponse {
        ResponseDomainService::create_saved_response(name, api_id, doc_content)
    }

    /// 创建索引条目（封装Domain服务）
    pub fn create_index_entry(saved_response: &SavedResponse) -> SavedResponseIndexEntry {
        ResponseDomainService::create_index_entry(saved_response)
    }

    /// 获取响应索引列表（格式化时间）
    pub fn get_all(workspace_id: &str) -> Result<Vec<SavedResponseIndexEntry>, String> {
        let repository = RepositoryFactory::get_response_repository();
        let index = repository.get_index(workspace_id)?;
        Ok(index
            .responses
            .into_iter()
            .map(|e| SavedResponseIndexEntry {
                id: e.id,
                name: e.name,
                created_at: Self::format_time(&e.created_at),
                api_id: e.api_id,
            })
            .collect())
    }

    /// 获取单个响应详情（格式化时间）
    pub fn get(workspace_id: &str, id: &str) -> Result<Option<SavedResponse>, String> {
        let repository = RepositoryFactory::get_response_repository();
        let result = repository.get(workspace_id, id)?;
        Ok(result.map(|r| SavedResponse {
            id: r.id,
            name: r.name,
            created_at: Self::format_time(&r.created_at),
            api_id: r.api_id,
            doc_content: r.doc_content,
        }))
    }

    /// 保存响应快照
    pub fn save(
        workspace_id: &str,
        response: &SavedResponse,
        index_entry: &SavedResponseIndexEntry,
    ) -> Result<(), String> {
        let repository = RepositoryFactory::get_response_repository();
        repository.save(workspace_id, response, index_entry)
    }

    /// 删除响应快照
    pub fn delete(workspace_id: &str, id: &str) -> Result<(), String> {
        let repository = RepositoryFactory::get_response_repository();
        repository.delete(workspace_id, id)
    }

    /// 获取指定 API 的响应列表（格式化时间）
    pub fn get_by_api(
        workspace_id: &str,
        api_id: &str,
    ) -> Result<Vec<SavedResponseIndexEntry>, String> {
        let repository = RepositoryFactory::get_response_repository();
        let entries = repository.filter_by_api(workspace_id, api_id)?;
        Ok(entries
            .into_iter()
            .map(|e| SavedResponseIndexEntry {
                id: e.id,
                name: e.name,
                created_at: Self::format_time(&e.created_at),
                api_id: e.api_id,
            })
            .collect())
    }
}
