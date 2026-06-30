//! 文档应用服务
//!
//! 处理文档相关的 UI 交互，通过仓储工厂动态获取仓储。

use crate::domain::models::DocMetadata;
use crate::infrastructure::RepositoryFactory;

/// 文档应用服务
///
/// 无状态服务，每次调用时通过工厂根据 workspace_id 获取对应仓储
pub struct MdApplicationService;

impl MdApplicationService {
    /// 创建默认实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for MdApplicationService {
    fn default() -> Self {
        Self::new()
    }
}

impl MdApplicationService {
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

    /// 获取 API 文档内容
    pub fn get_doc(workspace_id: &str, api_id: &str) -> Result<String, String> {
        let repository = RepositoryFactory::get_md_repository();
        repository.read_api_doc(workspace_id, api_id)
    }

    /// 保存 API 文档内容
    pub fn save_doc(workspace_id: &str, api_id: &str, content: &str) -> Result<(), String> {
        let repository = RepositoryFactory::get_md_repository();
        repository.write_api_doc(workspace_id, api_id, content)
    }

    /// 获取文档元数据
    pub fn get_metadata(workspace_id: &str, api_id: &str) -> Result<DocMetadata, String> {
        let repository = RepositoryFactory::get_md_repository();
        let entry = repository.get_doc_index_entry(workspace_id, api_id)?;
        Ok(DocMetadata {
            api_id: api_id.to_string(),
            updated_at: entry.map(|e| Self::format_time(&e.updated_at)),
        })
    }
}
