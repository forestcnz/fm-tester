use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    ChatSession, Cookie, Environment, HistoryEntry, Orchestration, OrchestrationRun, SavedResponse,
    StressTestResult,
};

/// 工作区导出文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceExport {
    pub version: String,
    pub exported_at: String,
    pub app_version: String,
    pub workspace: WorkspaceExportMeta,
    pub data: WorkspaceExportData,
}

impl WorkspaceExport {
    pub const CURRENT_VERSION: &'static str = "1.0";
}

/// 工作区元信息（不含 ID）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceExportMeta {
    pub name: String,
    pub description: String,
}

/// 工作区完整导出数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceExportData {
    pub environments: Vec<Environment>,
    pub active_environment_id: Option<String>,
    pub collection_items: Vec<CollectionExportItem>,
    pub scripts: Vec<ScriptExportItem>,
    pub saved_responses: Vec<SavedResponse>,
    pub cookies: Vec<Cookie>,
    pub history_entries: Vec<HistoryEntry>,
    pub orchestrations: Vec<Orchestration>,
    pub orchestration_runs: Vec<OrchestrationRun>,
    pub stress_configs: Vec<StressConfigExport>,
    pub stress_results: Vec<StressTestResult>,
    pub docs: Vec<DocExportItem>,
    pub chat_sessions: Vec<ChatSession>,
    pub app_state: Option<AppStateExport>,
    pub ws_configs: Vec<WsConfigItem>,
}

/// 集合导出项（扁平化存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionExportItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub item_type: String,
    pub parent_id: Option<String>,
    pub order_index: i32,
    pub method: Option<String>,
    pub url: Option<String>,
    pub body: Option<String>,
    pub body_type: Option<String>,
    pub params: Vec<ParamExport>,
    pub headers: Vec<HeaderExport>,
    pub form_fields: Vec<FormFieldExport>,
    pub common_headers: Vec<HeaderExport>,
    pub variables: Vec<VariableExport>,
    pub ws_config: Option<WsConfigExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamExport {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderExport {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormFieldExport {
    pub key: String,
    pub value: String,
    pub field_type: String,
    pub enabled: bool,
    pub files: Option<Vec<FileInfoExport>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfoExport {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableExport {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsConfigExport {
    pub url: Option<String>,
    pub headers: Vec<HeaderExport>,
    pub reconnect: bool,
    pub reconnect_interval: u64,
    pub max_reconnect_attempts: u32,
}

/// 脚本导出项（含内容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExportItem {
    pub target_type: String,
    pub target_id: Option<String>,
    pub script_kind: String,
    pub filename: String,
    pub content: String,
}

/// 压测断言
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionExport {
    pub field: String,
    pub operator: String,
    pub expected: String,
}

/// 压测配置导出项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressConfigExport {
    pub api_id: String,
    pub concurrent: u32,
    pub total_requests: Option<u64>,
    pub duration_seconds: Option<u32>,
    pub ramp_up_seconds: u32,
    pub timeout_ms: u64,
    #[serde(default)]
    pub assertions: Vec<AssertionExport>,
}

/// WebSocket 配置导出项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsConfigItem {
    pub id: String,
    pub name: String,
    pub url: String,
    pub headers: Vec<HeaderExport>,
    pub params: Vec<ParamExport>,
    pub created_at: String,
    pub updated_at: String,
}

/// 文档导出项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocExportItem {
    pub api_id: String,
    pub updated_at: String,
    pub content: String,
}

/// 应用状态导出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStateExport {
    pub expanded_ids: Vec<String>,
    pub open_tabs: Vec<OpenTabExport>,
    pub active_tab_index: usize,
    pub request_tabs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenTabExport {
    pub id: String,
    #[serde(rename = "type")]
    pub tab_type: String,
}

/// 导入预览信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceImportPreview {
    pub name: String,
    pub description: String,
    pub exported_at: String,
    pub app_version: String,
    pub stats: WorkspaceExportStats,
}

/// 导出数据统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceExportStats {
    pub environments: usize,
    pub collections: usize,
    pub apis: usize,
    pub websockets: usize,
    pub scripts: usize,
    pub saved_responses: usize,
    pub cookies: usize,
    pub history_entries: usize,
    pub orchestrations: usize,
    pub stress_results: usize,
    pub docs: usize,
    pub chat_sessions: usize,
    pub ws_configs: usize,
}

impl WorkspaceExportStats {
    pub fn total_items(&self) -> usize {
        self.collections + self.apis + self.websockets
    }
}
