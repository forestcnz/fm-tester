//! 编排命令接口
//!
//! 提供编排相关的 Tauri 命令，处理前端交互。

use crate::application::services::OrchestrationApplicationService;
use crate::domain::models::{
    Orchestration, OrchestrationIndex, OrchestrationRun, OrchestrationRunIndex,
    OrchestrationSchedule, OrchestrationStep, StepRunResult,
};
use crate::infrastructure::SchedulerService;
use tauri::AppHandle;

#[tauri::command]
pub fn get_orchestrations(workspace_id: String) -> Result<OrchestrationIndex, String> {
    OrchestrationApplicationService::get_orchestrations(&workspace_id)
}

#[tauri::command]
pub fn get_orchestration(
    workspace_id: String,
    orchestration_id: String,
) -> Result<Orchestration, String> {
    OrchestrationApplicationService::get_orchestration(&workspace_id, &orchestration_id)
}

#[tauri::command]
pub fn create_orchestration_cmd(
    workspace_id: String,
    name: String,
    description: Option<String>,
) -> Result<Orchestration, String> {
    OrchestrationApplicationService::create_orchestration(&workspace_id, name, description)
}

#[tauri::command]
pub fn update_orchestration_cmd(
    workspace_id: String,
    orchestration_id: String,
    name: Option<String>,
    description: Option<String>,
) -> Result<Orchestration, String> {
    OrchestrationApplicationService::update_orchestration(
        &workspace_id,
        &orchestration_id,
        name,
        description,
    )
}

#[tauri::command]
pub fn delete_orchestration_cmd(
    workspace_id: String,
    orchestration_id: String,
) -> Result<(), String> {
    OrchestrationApplicationService::delete_orchestration(&workspace_id, &orchestration_id)
}

#[tauri::command]
pub fn add_orchestration_step_cmd(
    workspace_id: String,
    orchestration_id: String,
    api_id: String,
    name: Option<String>,
    enabled: Option<bool>,
    wait_before: Option<u64>,
    retry_count: Option<u32>,
    retry_delay: Option<u64>,
    on_failure: Option<String>,
) -> Result<OrchestrationStep, String> {
    OrchestrationApplicationService::add_orchestration_step(
        &workspace_id,
        &orchestration_id,
        api_id,
        name,
        enabled.unwrap_or(true),
        wait_before.unwrap_or(0),
        retry_count.unwrap_or(0),
        retry_delay.unwrap_or(1000),
        on_failure.unwrap_or_else(|| "stop".to_string()),
    )
}

#[tauri::command]
pub fn update_orchestration_step_cmd(
    workspace_id: String,
    orchestration_id: String,
    step_id: String,
    name: Option<String>,
    enabled: Option<bool>,
    wait_before: Option<u64>,
    retry_count: Option<u32>,
    retry_delay: Option<u64>,
    on_failure: Option<String>,
) -> Result<OrchestrationStep, String> {
    OrchestrationApplicationService::update_orchestration_step(
        &workspace_id,
        &orchestration_id,
        &step_id,
        name,
        enabled,
        wait_before,
        retry_count,
        retry_delay,
        on_failure,
    )
}

#[tauri::command]
pub fn remove_orchestration_step_cmd(
    workspace_id: String,
    orchestration_id: String,
    step_id: String,
) -> Result<(), String> {
    OrchestrationApplicationService::remove_orchestration_step(
        &workspace_id,
        &orchestration_id,
        &step_id,
    )
}

#[tauri::command]
pub fn reorder_orchestration_steps_cmd(
    workspace_id: String,
    orchestration_id: String,
    step_ids: Vec<String>,
) -> Result<(), String> {
    OrchestrationApplicationService::reorder_orchestration_steps(
        &workspace_id,
        &orchestration_id,
        step_ids,
    )
}

#[tauri::command]
pub fn create_orchestration_run_cmd(
    workspace_id: String,
    orchestration_id: String,
) -> Result<OrchestrationRun, String> {
    OrchestrationApplicationService::create_orchestration_run(&workspace_id, &orchestration_id)
}

#[tauri::command]
pub fn update_orchestration_run_step_cmd(
    workspace_id: String,
    orchestration_id: String,
    run_id: String,
    step_result: StepRunResult,
) -> Result<(), String> {
    OrchestrationApplicationService::update_orchestration_run_step(
        &workspace_id,
        &orchestration_id,
        &run_id,
        step_result,
    )
}

#[tauri::command]
pub fn complete_orchestration_run_cmd(
    workspace_id: String,
    orchestration_id: String,
    run_id: String,
    status: String,
    end_time: String,
    total_time: u64,
) -> Result<OrchestrationRun, String> {
    OrchestrationApplicationService::complete_orchestration_run(
        &workspace_id,
        &orchestration_id,
        &run_id,
        status,
        end_time,
        total_time,
    )
}

#[tauri::command]
pub fn get_orchestration_runs(
    workspace_id: String,
    orchestration_id: String,
) -> Result<OrchestrationRunIndex, String> {
    OrchestrationApplicationService::get_orchestration_runs(&workspace_id, &orchestration_id)
}

#[tauri::command]
pub fn get_orchestration_run(
    workspace_id: String,
    orchestration_id: String,
    run_id: String,
) -> Result<OrchestrationRun, String> {
    OrchestrationApplicationService::get_orchestration_run(
        &workspace_id,
        &orchestration_id,
        &run_id,
    )
}

#[tauri::command]
pub fn delete_orchestration_run_cmd(
    workspace_id: String,
    orchestration_id: String,
    run_id: String,
) -> Result<(), String> {
    OrchestrationApplicationService::delete_orchestration_run(
        &workspace_id,
        &orchestration_id,
        &run_id,
    )
}

#[tauri::command]
pub fn clear_orchestration_runs_cmd(
    workspace_id: String,
    orchestration_id: String,
) -> Result<(), String> {
    OrchestrationApplicationService::clear_orchestration_runs(&workspace_id, &orchestration_id)
}

/// 更新编排调度配置
#[tauri::command]
pub async fn update_orchestration_schedule_cmd(
    app: AppHandle,
    workspace_id: String,
    orchestration_id: String,
    schedule: OrchestrationSchedule,
) -> Result<Orchestration, String> {
    // 克隆 schedule 用于后续检查
    let schedule_clone = schedule.clone();

    // 更新调度配置
    let orchestration = OrchestrationApplicationService::update_orchestration_schedule(
        &workspace_id,
        &orchestration_id,
        schedule,
    )?;

    // 启动或停止定时任务
    if schedule_clone.enabled {
        // 启动定时任务
        SchedulerService::start_task(app, workspace_id, orchestration.clone())?;
    } else {
        // 停止定时任务
        SchedulerService::stop_task(&orchestration_id)?;
    }

    Ok(orchestration)
}

/// 恢复所有定时任务（应用启动时调用）
#[tauri::command]
pub async fn restore_scheduled_tasks_cmd(
    app: AppHandle,
    workspace_id: String,
) -> Result<(), String> {
    SchedulerService::restore_all(app, &workspace_id)?;
    Ok(())
}

/// 获取运行中的定时任务列表
#[tauri::command]
pub fn get_scheduled_tasks_cmd() -> Result<Vec<String>, String> {
    SchedulerService::get_running_tasks()
}

/// 获取下次执行时间
#[tauri::command]
pub fn get_next_run_time_cmd(cron_expression: String) -> Result<String, String> {
    OrchestrationApplicationService::get_next_run_time(&cron_expression)
}

/// 获取最近 N 次执行时间
#[tauri::command]
pub fn get_next_run_times_cmd(
    cron_expression: String,
    count: usize,
) -> Result<Vec<String>, String> {
    OrchestrationApplicationService::get_next_run_times(&cron_expression, count)
}

/// 重排编排列表顺序
#[tauri::command]
pub fn reorder_orchestrations_cmd(
    workspace_id: String,
    orchestration_ids: Vec<String>,
) -> Result<(), String> {
    OrchestrationApplicationService::reorder_orchestrations(&workspace_id, orchestration_ids)
}
