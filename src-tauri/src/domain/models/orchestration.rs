use super::{generate_id, Header};
use serde::{Deserialize, Serialize};

/// 编排调度配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationSchedule {
    /// 是否启用定时任务
    #[serde(default)]
    pub enabled: bool,
    /// Cron 表达式
    pub cron_expression: String,
    /// 下次执行时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_run_at: Option<String>,
    /// 上次执行时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run_at: Option<String>,
    /// 执行次数
    #[serde(default)]
    pub run_count: u32,
}

impl OrchestrationSchedule {
    /// 验证调度配置
    pub fn validate(&self) -> Result<(), String> {
        // 验证 cron 表达式格式（基础验证）
        if self.cron_expression.trim().is_empty() {
            return Err("Cron 表达式不能为空".to_string());
        }

        // 验证 cron 表达式格式（5-6 个字段）
        let parts: Vec<&str> = self.cron_expression.split_whitespace().collect();
        if parts.len() < 5 || parts.len() > 6 {
            return Err(format!(
                "Cron 表达式格式错误，应为 5 或 6 个字段，实际为 {} 个字段",
                parts.len()
            ));
        }

        Ok(())
    }
}

impl Default for OrchestrationSchedule {
    fn default() -> Self {
        Self {
            enabled: false,
            cron_expression: "0 0 * * * *".to_string(), // 默认每小时执行一次
            next_run_at: None,
            last_run_at: None,
            run_count: 0,
        }
    }
}

/// 编排步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationStep {
    pub id: String,
    pub api_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub wait_before: u64,
    #[serde(default)]
    pub retry_count: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
    #[serde(default)]
    pub on_failure: String,
}

fn default_retry_delay() -> u64 {
    1000
}

fn default_true() -> bool {
    true
}

impl Default for OrchestrationStep {
    fn default() -> Self {
        Self {
            id: generate_id("step"),
            api_id: "".to_string(),
            name: None,
            enabled: true,
            wait_before: 0,
            retry_count: 0,
            retry_delay: 1000,
            on_failure: "stop".to_string(),
        }
    }
}

/// 编排定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orchestration {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub steps: Vec<OrchestrationStep>,
    /// 定时调度配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<OrchestrationSchedule>,
}

/// 编排索引条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationIndexEntry {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub step_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

/// 编排索引文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrchestrationIndex {
    pub orchestrations: Vec<OrchestrationIndexEntry>,
}

/// 步骤执行结果中的断言结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepTestResult {
    pub name: String,
    pub passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 步骤执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRunResult {
    pub step_id: String,
    pub api_id: String,
    pub api_name: String,
    pub status: String,
    pub start_time: String,
    pub end_time: String,
    pub response_time: u64,
    pub status_code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_body: Option<String>,
    #[serde(default)]
    pub response_headers: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub test_results: Vec<StepTestResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_message: Option<String>,
    pub retry_count: u32,
    #[serde(default)]
    pub request_method: String,
    #[serde(default)]
    pub request_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_original_url: Option<String>,
    #[serde(default)]
    pub request_headers: Vec<Header>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body_type: Option<String>,
}

/// 编排执行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationRun {
    pub id: String,
    pub orchestration_id: String,
    pub status: String,
    pub start_time: String,
    pub end_time: String,
    pub total_time: u64,
    pub success_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    pub steps: Vec<StepRunResult>,
}

/// 编排执行索引条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationRunIndexEntry {
    pub id: String,
    pub orchestration_id: String,
    pub status: String,
    pub start_time: String,
    pub total_time: u64,
    pub success_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
}

/// 编排执行索引文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrchestrationRunIndex {
    pub runs: Vec<OrchestrationRunIndexEntry>,
}

/// 编排执行记录集合（合并存储）
///
/// 存储在 `orchestrations/{id}.runs.toml` 文件中。
///
/// ## 设计理由
/// - 去掉 `{id}/runs/{run_id}.toml` 的嵌套层级
/// - 每个编排的所有执行记录合并到一个文件
/// - 便于查看单个编排的历史执行情况
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrchestrationRuns {
    pub runs: Vec<OrchestrationRun>,
}
