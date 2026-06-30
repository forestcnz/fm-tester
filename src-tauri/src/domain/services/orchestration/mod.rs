//! 编排领域服务
//!
//! 提供编排相关的业务逻辑，不依赖持久化实现。

use crate::domain::models::common::generate_id;
use crate::domain::models::{
    Orchestration, OrchestrationIndexEntry, OrchestrationRun, OrchestrationStep, StepRunResult,
};
use chrono::Local;

/// 编排领域服务
pub struct OrchestrationDomainService;

impl OrchestrationDomainService {
    /// 生成编排 ID
    pub fn generate_orchestration_id() -> String {
        generate_id("orch")
    }

    /// 生成步骤 ID
    pub fn generate_step_id() -> String {
        generate_id("step")
    }

    /// 生成执行记录 ID
    pub fn generate_run_id() -> String {
        generate_id("run")
    }

    /// 创建编排实体
    pub fn create_orchestration_entity(name: String, description: Option<String>) -> Orchestration {
        let now = Local::now().to_rfc3339();
        Orchestration {
            id: Self::generate_orchestration_id(),
            name,
            description,
            created_at: now.clone(),
            updated_at: now,
            steps: Vec::new(),
            schedule: None,
        }
    }

    /// 创建编排索引条目
    pub fn create_index_entry(orchestration: &Orchestration) -> OrchestrationIndexEntry {
        OrchestrationIndexEntry {
            id: orchestration.id.clone(),
            name: orchestration.name.clone(),
            description: orchestration.description.clone(),
            step_count: orchestration.steps.len(),
            created_at: orchestration.created_at.clone(),
            updated_at: orchestration.updated_at.clone(),
        }
    }

    /// 创建步骤实体
    pub fn create_step_entity(
        api_id: String,
        name: Option<String>,
        enabled: bool,
        wait_before: u64,
        retry_count: u32,
        retry_delay: u64,
        on_failure: String,
    ) -> OrchestrationStep {
        OrchestrationStep {
            id: Self::generate_step_id(),
            api_id,
            name,
            enabled,
            wait_before,
            retry_count,
            retry_delay,
            on_failure,
        }
    }

    /// 更新编排时间戳
    pub fn update_timestamp(orchestration: &mut Orchestration) {
        orchestration.updated_at = Local::now().to_rfc3339();
    }

    /// 验证编排名称
    pub fn validate_name(name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("编排名称不能为空".to_string());
        }
        Ok(())
    }

    /// 创建执行记录实体
    pub fn create_run_entity(orchestration_id: &str) -> OrchestrationRun {
        let now = Local::now().to_rfc3339();
        OrchestrationRun {
            id: Self::generate_run_id(),
            orchestration_id: orchestration_id.to_string(),
            status: "running".to_string(),
            start_time: now.clone(),
            end_time: "".to_string(),
            total_time: 0,
            success_count: 0,
            failed_count: 0,
            skipped_count: 0,
            steps: Vec::new(),
        }
    }

    /// 更新执行记录步骤状态统计
    pub fn update_run_statistics(run: &mut OrchestrationRun, step_result: &StepRunResult) {
        match step_result.status.as_str() {
            "success" => run.success_count += 1,
            "failed" => run.failed_count += 1,
            "skipped" => run.skipped_count += 1,
            _ => {}
        }
    }

    /// 完成执行记录
    pub fn complete_run(
        run: &mut OrchestrationRun,
        status: String,
        end_time: String,
        total_time: u64,
    ) {
        run.status = status;
        run.end_time = end_time;
        run.total_time = total_time;
    }

    /// 查找步骤
    pub fn find_step<'a>(
        orchestration: &'a mut Orchestration,
        step_id: &str,
    ) -> Option<&'a mut OrchestrationStep> {
        orchestration.steps.iter_mut().find(|s| s.id == step_id)
    }

    /// 重排步骤顺序
    pub fn reorder_steps(
        orchestration: &mut Orchestration,
        step_ids: &[String],
    ) -> Result<(), String> {
        let mut reordered_steps = Vec::new();
        for step_id in step_ids {
            let step = orchestration
                .steps
                .iter()
                .find(|s| s.id == *step_id)
                .ok_or(format!("步骤 {} 不存在", step_id))?
                .clone();
            reordered_steps.push(step);
        }
        orchestration.steps = reordered_steps;
        Ok(())
    }

    /// 添加步骤到编排
    pub fn add_step(orchestration: &mut Orchestration, step: OrchestrationStep) {
        orchestration.steps.push(step);
    }

    /// 移除步骤
    pub fn remove_step(orchestration: &mut Orchestration, step_id: &str) {
        orchestration.steps.retain(|s| s.id != step_id);
    }

    /// 更新步骤属性
    pub fn update_step(
        step: &mut OrchestrationStep,
        name: Option<String>,
        enabled: Option<bool>,
        wait_before: Option<u64>,
        retry_count: Option<u32>,
        retry_delay: Option<u64>,
        on_failure: Option<String>,
    ) {
        if let Some(n) = name {
            step.name = Some(n);
        }
        if let Some(e) = enabled {
            step.enabled = e;
        }
        if let Some(w) = wait_before {
            step.wait_before = w;
        }
        if let Some(r) = retry_count {
            step.retry_count = r;
        }
        if let Some(d) = retry_delay {
            step.retry_delay = d;
        }
        if let Some(f) = on_failure {
            step.on_failure = f;
        }
    }

    /// 添加或更新执行步骤结果
    pub fn add_or_update_step_result(run: &mut OrchestrationRun, step_result: StepRunResult) {
        let step_id = step_result.step_id.clone();
        let existing_index = run.steps.iter().position(|s| s.step_id == step_id);
        if let Some(idx) = existing_index {
            run.steps[idx] = step_result;
        } else {
            run.steps.push(step_result);
        }
    }
}
