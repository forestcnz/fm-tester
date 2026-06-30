use super::{Header, HttpResponse, Param, ScriptTargetType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 脚本执行上下文（输入）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionContext {
    /// 环境变量
    pub environment_variables: HashMap<String, String>,
    /// 集合变量（合并后的，用于变量替换）
    pub collection_variables: HashMap<String, String>,
    /// 所有集合变量（按集合 ID 分组）
    pub all_collection_variables: HashMap<String, HashMap<String, String>>,
    /// 当前脚本的目标集合 ID（用于集合脚本操作自己集合的变量）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_collection_id: Option<String>,
    /// 当前脚本的目标环境 ID（用于环境脚本操作环境变量）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_environment_id: Option<String>,
    /// 是否是 API 脚本（用于操作父集合变量）
    pub is_api_script: bool,
    /// API 的直接父集合 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_collection_id: Option<String>,
    /// 请求上下文
    pub request: ScriptRequestContext,
    /// 响应上下文（仅后置脚本有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<ScriptResponseContext>,
}

/// 请求上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptRequestContext {
    pub url: String,
    pub method: String,
    pub headers: Vec<Header>,
    #[serde(default)]
    pub params: Vec<Param>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

/// 响应上下文（只读）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResponseContext {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub time: u64,
    pub size: u64,
}

/// 从 HttpResponse 创建 ScriptResponseContext
impl From<HttpResponse> for ScriptResponseContext {
    fn from(response: HttpResponse) -> Self {
        ScriptResponseContext {
            status: response.status,
            status_text: response.status_text,
            headers: response.headers,
            body: response.body,
            time: response.time,
            size: response.size,
        }
    }
}

/// 脚本执行结果（输出）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionResult {
    /// 是否成功
    pub success: bool,
    /// 修改后的环境变量（合并后的）
    pub modified_environment_vars: HashMap<String, String>,
    /// 修改后的目标环境变量（仅环境脚本修改的）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_target_environment_vars: Option<HashMap<String, String>>,
    /// 目标环境 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_environment_id: Option<String>,
    /// 修改后的集合变量（合并后的，用于传递）
    pub modified_collection_vars: HashMap<String, String>,
    /// 修改后的目标集合变量（仅该集合脚本修改的）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_target_collection_vars: Option<HashMap<String, String>>,
    /// 目标集合 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_collection_id: Option<String>,
    /// 修改后的请求（仅前置脚本）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_request: Option<ScriptRequestContext>,
    /// 测试结果（仅后置脚本）
    #[serde(default)]
    pub test_results: Vec<ScriptTestResult>,
    /// 日志列表
    #[serde(default)]
    pub logs: Vec<ScriptLog>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 错误来源
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_source: Option<String>,
}

/// 测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptTestResult {
    /// 测试名称
    pub name: String,
    /// 是否通过
    pub passed: bool,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 脚本日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptLog {
    /// 日志级别 ("log" | "error")
    pub level: String,
    /// 日志内容
    pub message: String,
    /// 来源层级 ("workspace" | "environment" | "collection:xxx" | "api")
    pub source: String,
}

/// 脚本信息（用于脚本链执行）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptInfo {
    /// 来源层级 ("workspace" | "environment" | "collection:xxx" | "api")
    pub source: String,
    /// 脚本内容
    pub content: String,
    /// 目标类型
    pub source_type: ScriptTargetType,
    /// 目标 ID（环境ID、集合ID、或 None）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
}

/// 集合信息（用于传递集合层级）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub collection_variables: Vec<super::Variable>,
}

/// 脚本执行输入参数（Tauri 命令参数）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutePreScriptsInput {
    pub workspace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,
    pub ancestor_collections: Vec<CollectionInfo>,
    pub environment_variables: HashMap<String, String>,
    pub collection_variables: HashMap<String, String>,
    pub request: ScriptRequestContext,
    /// 是否静默模式
    #[serde(default)]
    pub silent: bool,
}

/// 脚本执行输入参数（Tauri 命令参数）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutePostScriptsInput {
    pub workspace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,
    pub ancestor_collections: Vec<CollectionInfo>,
    pub environment_variables: HashMap<String, String>,
    pub collection_variables: HashMap<String, String>,
    pub request: ScriptRequestContext,
    pub response: ScriptResponseContext,
    /// 是否静默模式
    #[serde(default)]
    pub silent: bool,
}
