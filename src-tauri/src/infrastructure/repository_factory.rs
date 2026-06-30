//! 仓储工厂
//!
//! 为应用层提供统一的仓储实例获取入口。
//! 应用配置使用 JSON 文件存储，工作区数据使用 SQLite 仓储实现。

use crate::domain::repositories::AppConfigRepository;
use crate::domain::repositories::{
    ChatRepository, CollectionRepository, HistoryRepository, MdRepository, OrchestrationRepository,
    ResponseRepository, ScriptRepository, StressTestRepository, WorkspaceDataRepository,
    WsConfigRepository,
};
use crate::infrastructure::sqlite::{
    SqliteChatRepository, SqliteCollectionRepository, SqliteHistoryRepository, SqliteMdRepository,
    SqliteOrchestrationRepository, SqliteResponseRepository, SqliteScriptRepository,
    SqliteStressRepository, SqliteWorkspaceDataRepository, SqliteWsConfigRepository,
};
use crate::infrastructure::JsonAppConfigRepository;

/// 仓储工厂
pub struct RepositoryFactory;

impl RepositoryFactory {
    /// 获取应用配置仓储（settings + workspaces）
    /// 使用 JSON 文件存储：./data/config.json
    pub fn get_app_config_repository() -> Box<dyn AppConfigRepository> {
        Box::new(JsonAppConfigRepository::new().expect("无法初始化应用配置仓储"))
    }

    /// 获取工作区数据仓储（environments / memory / cookies）
    pub fn get_workspace_data_repository() -> Box<dyn WorkspaceDataRepository> {
        Box::new(SqliteWorkspaceDataRepository::new())
    }

    /// 获取集合仓储
    pub fn get_collection_repository() -> Box<dyn CollectionRepository> {
        Box::new(SqliteCollectionRepository::new())
    }

    /// 获取 History 仓储
    pub fn get_history_repository() -> Box<dyn HistoryRepository> {
        Box::new(SqliteHistoryRepository::new())
    }

    /// 获取 Response 仓储
    pub fn get_response_repository() -> Box<dyn ResponseRepository> {
        Box::new(SqliteResponseRepository::new())
    }

    /// 获取 Script 仓储
    pub fn get_script_repository() -> Box<dyn ScriptRepository> {
        Box::new(SqliteScriptRepository::new())
    }

    /// 获取 Chat 仓储
    pub fn get_chat_repository() -> Box<dyn ChatRepository> {
        Box::new(SqliteChatRepository::new())
    }

    /// 获取 Orchestration 仓储
    pub fn get_orchestration_repository() -> Box<dyn OrchestrationRepository> {
        Box::new(SqliteOrchestrationRepository::new())
    }

    /// 获取 Stress 仓储
    pub fn get_stress_repository() -> Box<dyn StressTestRepository> {
        Box::new(SqliteStressRepository::new())
    }

    /// 获取 Md 仓储
    pub fn get_md_repository() -> Box<dyn MdRepository> {
        Box::new(SqliteMdRepository::new())
    }

    /// 获取 WebSocket 配置仓储（ws_configs 表）
    pub fn get_ws_config_repository() -> Box<dyn WsConfigRepository> {
        Box::new(SqliteWsConfigRepository::new())
    }
}
