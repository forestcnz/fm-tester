//! 工作区数据仓储接口
//!
//! environments / memory / cookies 都持久化到工作区 SQLite 数据库的
//! 对应表中（早期版本曾使用过独立 .toml 文件，已迁移至 SQLite）。
//!
//! ## 对应表
//! - environments — 环境配置（业务数据）
//! - app_state — UI 状态（展开/标签页等）
//! - cookies — Cookie 数据

use crate::domain::models::{Cookie, CookiesConfig, Environment, EnvironmentsConfig, MemoryConfig};

/// 工作区数据仓储接口
///
/// 负责工作区级别数据的持久化操作，包括：
/// - 环境配置读写（environments 表）
/// - 记忆状态读写（app_state 表）
/// - Cookie 管理读写（cookies 表）
pub trait WorkspaceDataRepository: Send {
    // === 环境相关 ===

    /// 读取环境配置
    fn read_environments(&self, workspace_id: &str) -> Result<EnvironmentsConfig, String>;

    /// 写入环境配置
    fn write_environments(
        &self,
        workspace_id: &str,
        config: &EnvironmentsConfig,
    ) -> Result<(), String>;

    /// 查找环境（按名称）
    fn find_environment_by_name(
        &self,
        workspace_id: &str,
        name: &str,
    ) -> Result<Option<Environment>, String>;

    // === 记忆相关 ===

    /// 读取记忆配置
    fn read_memory(&self, workspace_id: &str) -> Result<MemoryConfig, String>;

    /// 写入记忆配置
    fn write_memory(&self, workspace_id: &str, config: &MemoryConfig) -> Result<(), String>;

    // === Cookie 相关 ===

    /// 读取 Cookie 配置
    fn read_cookies(&self, workspace_id: &str) -> Result<CookiesConfig, String>;

    /// 写入 Cookie 配置
    fn write_cookies(&self, workspace_id: &str, config: &CookiesConfig) -> Result<(), String>;

    /// 获取所有 Cookie
    fn get_all_cookies(&self, workspace_id: &str) -> Result<Vec<Cookie>, String>;

    /// 添加或更新 Cookie
    fn add_or_update_cookie(&self, workspace_id: &str, cookie: &Cookie) -> Result<(), String>;

    /// 删除 Cookie
    fn delete_cookie(&self, workspace_id: &str, name: &str, domain: &str) -> Result<(), String>;

    /// 清除所有 Cookie
    fn clear_cookies(&self, workspace_id: &str) -> Result<(), String>;
}
