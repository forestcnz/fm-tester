//! 文档仓储接口
//!
//! 定义文档相关数据的持久化接口，符合DDD依赖反转原则。

use crate::domain::models::{DocIndex, DocIndexEntry};

/// 文档仓储接口
pub trait MdRepository {
    /// 读取文档索引
    fn read_doc_index(&self, workspace_id: &str) -> Result<DocIndex, String>;

    /// 写入文档索引
    fn write_doc_index(&self, workspace_id: &str, index: &DocIndex) -> Result<(), String>;

    /// 更新文档索引（添加或更新条目）
    fn update_doc_index(
        &self,
        workspace_id: &str,
        api_id: &str,
        updated_at: &str,
    ) -> Result<(), String>;

    /// 获取文档索引条目
    fn get_doc_index_entry(
        &self,
        workspace_id: &str,
        api_id: &str,
    ) -> Result<Option<DocIndexEntry>, String>;

    /// 读取 API 文档
    fn read_api_doc(&self, workspace_id: &str, api_id: &str) -> Result<String, String>;

    /// 写入 API 文档
    fn write_api_doc(&self, workspace_id: &str, api_id: &str, content: &str) -> Result<(), String>;
}
