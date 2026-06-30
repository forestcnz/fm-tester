//! History 仓储接口
//!
//! 定义历史记录的数据访问抽象接口。
//! 历史记录按日期分目录存储：history/{YYYY-MM-DD}/{id}.toml

use crate::domain::models::HistoryEntry;

/// History 仓储接口
pub trait HistoryRepository {
    /// 获取所有有历史记录的日期列表
    fn list_dates(&self, workspace_id: &str) -> Result<Vec<String>, String>;

    /// 获取指定日期的历史记录列表（简要信息）
    fn get_by_date(&self, workspace_id: &str, date: &str) -> Result<Vec<HistoryEntry>, String>;

    /// 获取指定接口的最近历史记录（按 created_at 倒序，限制条数）
    fn get_by_api(
        &self,
        workspace_id: &str,
        api_id: &str,
        limit: usize,
    ) -> Result<Vec<HistoryEntry>, String>;

    /// 读取单个历史记录详情
    fn get_entry(
        &self,
        workspace_id: &str,
        date: &str,
        id: &str,
    ) -> Result<Option<HistoryEntry>, String>;

    /// 保存历史记录（自动按日期分组，更新索引）
    fn save_entry(&self, workspace_id: &str, entry: &HistoryEntry) -> Result<(), String>;

    /// 删除历史记录（更新索引）
    fn delete_entry(&self, workspace_id: &str, date: &str, id: &str) -> Result<(), String>;

    /// 清空指定日期的历史记录
    fn clear_by_date(&self, workspace_id: &str, date: &str) -> Result<(), String>;

    /// 清空所有历史记录
    fn clear_all(&self, workspace_id: &str) -> Result<(), String>;
}
