//! 领域层仓储接口定义
//!
//! 根据 DDD 依赖反转原则，仓储接口应在领域层定义，
//! 具体实现在基础设施层，这样领域层可以依赖抽象而非具体实现。

pub mod ai_http_client_repository;
pub mod app_config_repository; // 合并 settings + workspace
pub mod chat_repository;
pub mod collection_repository;
pub mod history_repository;
pub mod md_repository;
pub mod orchestration_repository;
pub mod response_repository;
pub mod script_repository;
pub mod stress_repository;
pub mod workspace_data_repository;
pub mod ws_config_repository; // WebSocket 配置（ws_configs 表） // environments / app_state / cookies 共用一个文件

pub use ai_http_client_repository::AiHttpClientService;
pub use app_config_repository::AppConfigRepository;
pub use chat_repository::ChatRepository;
pub use collection_repository::CollectionRepository;
pub use history_repository::HistoryRepository;
pub use md_repository::MdRepository;
pub use orchestration_repository::OrchestrationRepository;
pub use response_repository::ResponseRepository;
pub use script_repository::ScriptRepository;
pub use stress_repository::StressTestRepository;
pub use workspace_data_repository::WorkspaceDataRepository;
pub use ws_config_repository::WsConfigRepository;
