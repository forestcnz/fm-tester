//! 领域服务模块
//!
//! 提供纯业务逻辑，不依赖基础设施实现。

pub mod ai_domain;
pub mod app_config_domain; // 合并 settings + workspace
pub mod chat_domain;
pub mod collection_domain;
pub mod encryption_domain;
pub mod history_domain;
pub mod http_domain;
pub mod import;
pub mod orchestration;
pub mod response_domain;
pub mod script_domain;
pub mod script_execution_domain;
pub mod stress;
pub mod validation_domain;
pub mod workspace_data_domain; // 合并 environment + memory + cookie

pub use ai_domain::AiDomainService;
pub use app_config_domain::AppConfigDomainService;
pub use chat_domain::ChatDomainService;
pub use collection_domain::CollectionDomainService;
pub use collection_domain::{
    find_ancestor_chain, find_api_in_collections, find_collection_item, find_item_in_collections,
    find_parent_children, get_all_descendant_ids, get_collection_depth,
    get_collection_max_child_depth, remove_collection_item,
};
pub use encryption_domain::EncryptionService;
pub use history_domain::HistoryDomainService;
pub use http_domain::{parse_set_cookie, shell_escape};
pub use import::{
    convert_collection_to_postman, convert_postman_to_collection, convert_to_collection,
    parse_curl_command, parse_openapi, parse_postman,
};
pub use orchestration::OrchestrationDomainService;
pub use response_domain::ResponseDomainService;
pub use script_domain::ScriptDomainService;
pub use script_execution_domain::ScriptExecutionDomainService;
pub use workspace_data_domain::WorkspaceDataDomainService;
