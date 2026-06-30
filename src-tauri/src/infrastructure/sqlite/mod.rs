//! SQLite 基础设施层模块
//!
//! 为工作区提供 SQLite 数据库存储实现。
//! 工作区数据存储在 ./data/data_<workspace_id>.db 中。

pub mod connection;
pub mod schema;
pub mod sqlite_chat_repository;
pub mod sqlite_collection_repository;
pub mod sqlite_history_repository;
pub mod sqlite_md_repository;
pub mod sqlite_orchestration_repository;
pub mod sqlite_response_repository;
pub mod sqlite_script_repository;
pub mod sqlite_stress_repository;
pub mod sqlite_websocket_repository;
pub mod sqlite_workspace_data_repository;

pub use sqlite_chat_repository::SqliteChatRepository;
pub use sqlite_collection_repository::SqliteCollectionRepository;
pub use sqlite_history_repository::SqliteHistoryRepository;
pub use sqlite_md_repository::SqliteMdRepository;
pub use sqlite_orchestration_repository::SqliteOrchestrationRepository;
pub use sqlite_response_repository::SqliteResponseRepository;
pub use sqlite_script_repository::SqliteScriptRepository;
pub use sqlite_stress_repository::SqliteStressRepository;
pub use sqlite_websocket_repository::SqliteWsConfigRepository;
pub use sqlite_workspace_data_repository::SqliteWorkspaceDataRepository;
