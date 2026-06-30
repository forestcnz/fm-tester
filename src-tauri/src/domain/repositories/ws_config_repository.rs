//! WebSocket 配置仓储接口
//!
//! 定义 ws_configs 表对应数据的持久化接口，遵循 DDD 依赖反转原则。

use crate::domain::models::WsConfigEntry;

/// WebSocket 配置仓储接口
pub trait WsConfigRepository: Send {
    /// 读取所有 WebSocket 配置（按 order_index 升序）
    fn list(&self, workspace_id: &str) -> Result<Vec<WsConfigEntry>, String>;

    /// 保存（新建或更新）一个 WebSocket 配置。
    /// - 若 `entry.id` 在表中已存在，则更新对应行，返回相同 id。
    /// - 否则插入新行，使用 `entry.id` 作为主键。
    fn upsert(&self, workspace_id: &str, entry: &WsConfigEntry) -> Result<String, String>;

    /// 删除指定 id 的 WebSocket 配置
    fn delete(&self, workspace_id: &str, id: &str) -> Result<(), String>;
}
