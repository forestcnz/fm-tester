//! 基础设施层模块
//!
//! 提供持久化实现、HTTP 客户端、加密服务等基础设施。

pub mod ai_http_client;
pub mod auto_backup;
pub mod data_dir;
pub mod encryption;
pub mod git_backup;
pub mod http_client;
pub mod js_runtime;
pub mod json_app_config_repository;
pub mod logging;
pub mod repository_factory;
pub mod safe_file_ops;
pub mod scheduler;
pub mod sqlite;
pub mod sse_client;
pub mod ws_client;

pub use ai_http_client::get_ai_http_client;
pub use encryption::get_encryption_service;
pub use http_client::HttpClientService;
pub use js_runtime::JsRuntimeExecutor;
pub use json_app_config_repository::JsonAppConfigRepository;
pub use repository_factory::RepositoryFactory;
pub use scheduler::SchedulerService;
pub use sse_client::get_sse_client;
