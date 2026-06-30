//! Response 仓储接口
//!
//! 定义响应快照的数据访问抽象接口。
//! 存储结构：responses/{id}.toml + responses/index.toml

use crate::domain::models::{SavedResponse, SavedResponseIndexEntry, SavedResponsesIndex};

/// Response 仓储接口
pub trait ResponseRepository {
    /// 获取响应索引
    fn get_index(&self, workspace_id: &str) -> Result<SavedResponsesIndex, String>;

    /// 保存响应索引
    fn save_index(&self, workspace_id: &str, index: &SavedResponsesIndex) -> Result<(), String>;

    /// 获取单个响应详情
    fn get(&self, workspace_id: &str, id: &str) -> Result<Option<SavedResponse>, String>;

    /// 保存响应（同时更新索引）
    fn save(
        &self,
        workspace_id: &str,
        response: &SavedResponse,
        index_entry: &SavedResponseIndexEntry,
    ) -> Result<(), String>;

    /// 删除响应（同时更新索引）
    fn delete(&self, workspace_id: &str, id: &str) -> Result<(), String>;

    /// 按 api_id 过滤响应索引
    fn filter_by_api(
        &self,
        workspace_id: &str,
        api_id: &str,
    ) -> Result<Vec<SavedResponseIndexEntry>, String>;
}
