use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::models::{FormField, Header, Param, Variable};

/// 压测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestConfig {
    pub id: String,
    pub api_id: Option<String>,
    pub api_name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<Header>,
    #[serde(default)]
    pub params: Vec<Param>,
    pub body: Option<String>,
    pub body_type: Option<String>,
    pub form_fields: Option<Vec<FormField>>,
    pub collection_variables: Option<Vec<Variable>>,
    pub concurrent: u32,
    pub total_requests: Option<u64>,
    pub duration_seconds: Option<u32>,
    pub ramp_up_seconds: u32,
    pub timeout_ms: u64,
    /// 环境ID（用于加载脚本）
    pub environment_id: Option<String>,
    /// 祖先集合（用于加载集合脚本）
    pub ancestor_collections: Option<Vec<AncestorCollectionInfo>>,
}

/// 祖先集合信息（用于加载脚本）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AncestorCollectionInfo {
    pub id: String,
    pub name: String,
}

/// 失败请求记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedRequest {
    pub time: String,
    pub error: String,
    pub status: Option<u16>,
    pub elapsed_ms: u64,
}

/// 压测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResult {
    pub id: String,
    pub api_id: Option<String>,
    pub config: StressTestConfig,
    pub start_time: String,
    pub end_time: Option<String>,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_time_ms: u64,
    pub qps: f64,
    pub avg_time_ms: f64,
    pub min_time_ms: u64,
    pub max_time_ms: u64,
    pub p50_time_ms: f64,
    pub p90_time_ms: f64,
    pub p95_time_ms: f64,
    pub p99_time_ms: f64,
    pub success_rate: f64,
    // TOML requires all map keys to be strings, so we use String instead of u16
    pub status_distribution: HashMap<String, u64>,
    pub error_distribution: HashMap<String, u64>,
    #[serde(default)]
    pub failed_request_details: Vec<FailedRequest>,
    #[serde(default)]
    pub history: Vec<HistoryPoint>,
}

/// 历史数据点（每秒记录）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryPoint {
    pub second: u32,
    pub qps: f64,
    pub avg_time_ms: f64,
    pub successful: u64,
    pub failed: u64,
    pub requests: u64,
    pub concurrent: u32,
}

/// 实时进度
#[derive(Debug, Clone, Serialize)]
pub struct StressTestProgress {
    pub id: String,
    pub elapsed_seconds: u32,
    pub completed_requests: u64,
    pub current_qps: f64,
    pub current_avg_time_ms: f64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub current_concurrent: u32,
    pub is_running: bool,
    pub history: Vec<HistoryPoint>,
}

/// 压测结果索引条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResultIndexEntry {
    pub id: String,
    pub api_id: Option<String>,
    pub api_name: String,
    pub start_time: String,
    pub qps: f64,
    pub success_rate: f64,
    pub total_requests: u64,
}

/// 压测结果索引文件
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StressTestResultsIndex {
    pub results: Vec<StressTestResultIndexEntry>,
}

/// 接口压测参数配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressParamsConfig {
    pub concurrent: u32,
    pub total_requests: Option<u64>,
    pub duration_seconds: Option<u32>,
    pub ramp_up_seconds: u32,
    pub timeout_ms: u64,
}

impl Default for StressParamsConfig {
    fn default() -> Self {
        Self {
            concurrent: 10,
            total_requests: Some(100),
            duration_seconds: None,
            ramp_up_seconds: 0,
            timeout_ms: 30000,
        }
    }
}

/// API 压测配置（参数）
///
/// 存储在 `stress/{api_id}.toml` 文件中。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StressConfig {
    #[serde(default = "default_stress_params")]
    pub params: StressParamsConfig,
}

fn default_stress_params() -> StressParamsConfig {
    StressParamsConfig::default()
}

/// API 压测结果集合（合并存储）
///
/// 存储在 `stress/{api_id}.runs.toml` 文件中。
///
/// ## 设计理由
/// - 去掉 `{api_id}/results/{id}.toml` 的嵌套层级
/// - 每个API的所有压测结果合并到一个文件
/// - 便于查看单个API的历史压测情况
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StressResults {
    pub results: Vec<StressTestResult>,
}
