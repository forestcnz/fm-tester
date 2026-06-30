//! 应用配置仓储接口
//!
//! 全局配置存储在 ./data/config.db，包含 settings + workspaces。

use crate::domain::models::AppConfig;

/// 应用配置仓储接口（DDD 依赖反转）
pub trait AppConfigRepository: Send {
    /// 读取全局配置
    fn read(&self) -> Result<AppConfig, String>;

    /// 写入全局配置
    fn write(&self, config: &AppConfig) -> Result<(), String>;
}
