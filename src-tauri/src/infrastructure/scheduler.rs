//! 编排调度器
//!
//! 提供定时任务调度功能，支持 Quartz 格式 cron 表达式。

use crate::application::services::OrchestrationApplicationService;
use crate::domain::models::Orchestration;
use chrono::{DateTime, Utc};
use croner::Cron;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// 定时任务
pub struct ScheduledTask {
    /// 停止信号
    pub stop_signal: Arc<AtomicBool>,
}

lazy_static! {
    /// 全局定时任务映射
    static ref SCHEDULED_TASKS: Mutex<HashMap<String, ScheduledTask>> =
        Mutex::new(HashMap::new());
}

use std::sync::Mutex;

/// 调度器服务
pub struct SchedulerService;

impl SchedulerService {
    /// 启动定时任务
    ///
    /// # 参数
    /// - `app`: Tauri AppHandle，用于发送事件
    /// - `workspace_id`: 工作区路径
    /// - `orchestration`: 编排配置
    ///
    /// # 返回
    /// - `Ok(())`: 启动成功
    /// - `Err(String)`: 启动失败
    pub fn start_task(
        app: AppHandle,
        workspace_id: String,
        orchestration: Orchestration,
    ) -> Result<(), String> {
        let schedule_config = orchestration
            .schedule
            .as_ref()
            .ok_or("编排未配置定时任务")?;

        if !schedule_config.enabled {
            return Err("定时任务未启用".to_string());
        }

        // 验证 cron 表达式
        schedule_config.validate()?;

        let orchestration_id = orchestration.id.clone();
        let cron_expr = schedule_config.cron_expression.clone();

        // 解析 cron 表达式
        let schedule = Self::parse_cron_expression(&cron_expr)?;

        // 计算下次执行时间
        let next_run = Self::get_next_run_time(&schedule)?;
        let next_run_str = next_run.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string());

        // 停止已存在的任务
        Self::stop_task(&orchestration_id)?;

        // 创建停止信号
        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop_signal_clone = stop_signal.clone();

        // 克隆必要数据
        let app_clone = app.clone();
        let workspace_id_clone = workspace_id.clone();
        let orchestration_id_clone = orchestration_id.clone();

        // 启动后台任务
        tokio::spawn(async move {
            Self::run_scheduled_task(
                app_clone,
                workspace_id_clone,
                orchestration_id_clone,
                schedule,
                stop_signal_clone,
            )
            .await;
        });

        // 注册任务
        {
            let mut tasks = SCHEDULED_TASKS
                .lock()
                .map_err(|e| format!("获取任务锁失败: {}", e))?;
            tasks.insert(orchestration_id.clone(), ScheduledTask { stop_signal });
        }

        // 更新下次执行时间
        if let Some(next_run_str) = next_run_str {
            if let Err(e) =
                Self::update_next_run_time(&workspace_id, &orchestration_id, &next_run_str)
            {
                eprintln!("更新下次执行时间失败: {}", e);
            }
        }

        Ok(())
    }

    /// 停止定时任务
    ///
    /// # 参数
    /// - `orchestration_id`: 编排 ID
    ///
    /// # 返回
    /// - `Ok(())`: 停止成功
    /// - `Err(String)`: 停止失败
    pub fn stop_task(orchestration_id: &str) -> Result<(), String> {
        let mut tasks = SCHEDULED_TASKS
            .lock()
            .map_err(|e| format!("获取任务锁失败: {}", e))?;

        if let Some(task) = tasks.remove(orchestration_id) {
            task.stop_signal.store(true, Ordering::SeqCst);
            // 任务会在下一个循环周期自动退出
        }

        Ok(())
    }

    /// 恢复所有定时任务
    ///
    /// # 参数
    /// - `app`: Tauri AppHandle
    /// - `workspace_id`: 工作区路径
    ///
    /// # 返回
    /// - `Ok(())`: 恢复成功
    /// - `Err(String)`: 恢复失败
    pub fn restore_all(app: AppHandle, workspace_id: &str) -> Result<(), String> {
        // 获取所有编排
        let index = OrchestrationApplicationService::get_orchestrations(workspace_id)?;

        for entry in index.orchestrations {
            // 读取编排详情
            if let Ok(orchestration) =
                OrchestrationApplicationService::get_orchestration(workspace_id, &entry.id)
            {
                // 检查是否启用定时任务
                if let Some(ref schedule) = orchestration.schedule {
                    if schedule.enabled {
                        // 启动定时任务
                        if let Err(e) =
                            Self::start_task(app.clone(), workspace_id.to_string(), orchestration)
                        {
                            eprintln!("恢复定时任务 {} 失败: {}", entry.id, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 获取所有运行中的定时任务
    ///
    /// # 返回
    /// - 返回所有运行中的任务 ID 列表
    pub fn get_running_tasks() -> Result<Vec<String>, String> {
        let tasks = SCHEDULED_TASKS
            .lock()
            .map_err(|e| format!("获取任务锁失败: {}", e))?;

        Ok(tasks.keys().cloned().collect())
    }

    /// 解析 cron 表达式（Quartz 格式）
    fn parse_cron_expression(cron_expr: &str) -> Result<Cron, String> {
        // 使用 Quartz 格式解析（支持 ? 符号，周几 1-7）
        Cron::new(cron_expr)
            .with_alternative_weekdays() // Quartz 格式：周几 1-7（周日=1）
            .with_seconds_required() // 要求秒字段（6位格式）
            .parse()
            .map_err(|e| format!("Cron 表达式解析失败 (Quartz 格式): {}", e))
    }

    /// 获取下次执行时间
    fn get_next_run_time(cron: &Cron) -> Result<Option<DateTime<Utc>>, String> {
        let now = Utc::now();
        let next = cron.find_next_occurrence(&now, false);
        Ok(next.ok())
    }

    /// 运行定时任务
    async fn run_scheduled_task(
        app: AppHandle,
        _workspace_id: String,
        orchestration_id: String,
        cron: Cron,
        stop_signal: Arc<AtomicBool>,
    ) {
        loop {
            // 检查停止信号
            if stop_signal.load(Ordering::SeqCst) {
                break;
            }

            // 计算下次执行时间
            let now = Utc::now();
            let next_run = cron.find_next_occurrence(&now, false);

            if let Ok(next_time) = next_run {
                // 计算等待时间（毫秒精度）
                let duration: chrono::Duration = next_time - now;
                let wait_ms = duration.num_milliseconds();

                // 如果等待时间为负数或零，说明已经过了执行时间，跳过本次循环重新计算
                if wait_ms <= 0 {
                    // 短暂休眠 100ms 避免空转，然后重新计算
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }

                // 等待到下次执行时间（毫秒精度）
                tokio::time::sleep(Duration::from_millis(wait_ms as u64)).await;

                // 再次检查停止信号
                if stop_signal.load(Ordering::SeqCst) {
                    break;
                }

                // 执行编排
                // 发送事件通知前端
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    app.emit("orchestration-scheduled-run", &orchestration_id)
                }));
                if let Err(e) = result {
                    eprintln!("发送事件失败: {:?}", e);
                }
            } else {
                // 无法计算下次执行时间，退出循环
                break;
            }
        }
    }

    /// 更新下次执行时间
    fn update_next_run_time(
        workspace_id: &str,
        orchestration_id: &str,
        next_run_at: &str,
    ) -> Result<(), String> {
        let mut orchestration =
            OrchestrationApplicationService::get_orchestration(workspace_id, orchestration_id)?;

        if let Some(ref mut schedule) = orchestration.schedule {
            schedule.next_run_at = Some(next_run_at.to_string());
        }

        // 保存更新
        let repo = crate::infrastructure::RepositoryFactory::get_orchestration_repository();
        repo.write_orchestration(workspace_id, &orchestration)?;

        Ok(())
    }
}
