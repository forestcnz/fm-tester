//! 自动备份调度器
//!
//! 按设置在每天固定时刻自动备份所有工作区到 Git 仓库。
//! 借鉴 scheduler.rs 的 tokio::spawn + stop_signal 模式，但独立于编排调度。

use crate::domain::models::Workspace;
use crate::infrastructure::{git_backup, RepositoryFactory};
use chrono::{Local, NaiveTime, TimeZone};
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

lazy_static! {
    /// 自动备份后台任务的停止信号
    static ref AUTO_BACKUP_STOP: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);
}

pub struct AutoBackupScheduler;

impl AutoBackupScheduler {
    /// 启动或重启自动备份任务（读取最新设置决定是否启用）
    ///
    /// 在应用启动及设置变更后调用：先停止旧任务，再按最新设置决定是否启动。
    pub fn start(app: AppHandle) {
        // 先停止旧任务
        Self::stop();

        // 读取设置
        let config = match RepositoryFactory::get_app_config_repository().read() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("读取配置失败，自动备份未启动: {}", e);
                return;
            }
        };
        let g = &config.settings.git_backup;
        if !g.auto_backup_enabled {
            return;
        }
        if !g.is_configured() {
            tracing::warn!("自动备份已开启但未配置 Git 仓库，跳过启动");
            return;
        }

        let backup_time = g.auto_backup_time.clone();
        let stop_signal = Arc::new(AtomicBool::new(false));
        if let Ok(mut guard) = AUTO_BACKUP_STOP.lock() {
            *guard = Some(stop_signal.clone());
        }

        tracing::info!("自动备份已启动，计划每日 {} 执行", backup_time);
        tauri::async_runtime::spawn(async move {
            Self::run(app, backup_time, stop_signal).await;
        });
    }

    /// 停止自动备份任务
    pub fn stop() {
        if let Ok(mut guard) = AUTO_BACKUP_STOP.lock() {
            if let Some(signal) = guard.take() {
                signal.store(true, Ordering::SeqCst);
            }
        }
    }

    /// 后台循环：等待到每日备份时刻后执行，循环往复
    async fn run(app: AppHandle, backup_time: String, stop_signal: Arc<AtomicBool>) {
        loop {
            if stop_signal.load(Ordering::SeqCst) {
                break;
            }

            // 计算到下个备份时刻的等待时长
            let wait_ms = match Self::next_wait_ms(&backup_time) {
                Some(ms) => ms,
                None => {
                    // 时间格式无效，休眠后重试（等待设置被修正）
                    tokio::time::sleep(Duration::from_secs(3600)).await;
                    continue;
                }
            };

            // 分段睡眠（每分钟检查一次停止信号，便于及时退出）
            let mut remaining = wait_ms;
            while remaining > 0 {
                if stop_signal.load(Ordering::SeqCst) {
                    return;
                }
                let step = remaining.min(60_000);
                tokio::time::sleep(Duration::from_millis(step)).await;
                remaining -= step;
            }

            if stop_signal.load(Ordering::SeqCst) {
                break;
            }

            // 到点执行备份
            Self::backup_all(&app).await;
        }
    }

    /// 备份被选中的工作区
    ///
    /// 仅备份 `auto_backup_workspace_ids` 中存在的工作区；若列表为空则跳过全部，
    /// 不备份任何工作区。已选择但被删除的工作区会被自然忽略。
    async fn backup_all(app: &AppHandle) {
        tracing::info!("开始执行自动备份");

        // 一次性读取配置：工作区列表 + 自动备份目标工作区 id 集合
        let (workspaces, selected_ids): (Vec<Workspace>, HashSet<String>) =
            match RepositoryFactory::get_app_config_repository().read() {
                Ok(config) => {
                    let ids = config
                        .settings
                        .git_backup
                        .auto_backup_workspace_ids
                        .into_iter()
                        .collect();
                    (config.workspaces, ids)
                }
                Err(e) => {
                    tracing::error!("自动备份读取工作区列表失败: {}", e);
                    return;
                }
            };

        // 仅备份被勾选且仍存在的工作区
        let targets: Vec<&Workspace> = workspaces
            .iter()
            .filter(|w| selected_ids.contains(&w.id))
            .collect();

        if targets.is_empty() {
            tracing::warn!("自动备份未选择任何工作区，跳过执行");
        }

        let total = targets.len();
        let mut success = 0usize;
        for ws in targets {
            let ws_id = ws.id.clone();
            let ws_name = ws.name.clone();
            // git2 为阻塞操作，在独立线程执行
            match tokio::task::spawn_blocking(move || git_backup::backup_workspace(&ws_id)).await {
                Ok(Ok(_)) => {
                    success += 1;
                    tracing::info!("自动备份工作区成功: {}", ws_name);
                }
                Ok(Err(e)) => {
                    tracing::error!("自动备份工作区失败 [{}]: {}", ws_name, e);
                }
                Err(e) => {
                    tracing::error!("自动备份任务异常 [{}]: {}", ws_name, e);
                }
            }
        }

        // 通知前端备份完成（前端可选监听）
        let _ = app.emit("auto-backup-completed", (success, total));
        tracing::info!("自动备份完成: {}/{} 成功", success, total);
    }

    /// 计算到下个备份时刻（HH:MM）的毫秒数
    fn next_wait_ms(time_str: &str) -> Option<u64> {
        let target = NaiveTime::parse_from_str(time_str, "%H:%M").ok()?;
        let now = Local::now();
        let today_target = Local
            .from_local_datetime(&now.date_naive().and_time(target))
            .single()?;

        let next = if today_target > now {
            today_target
        } else {
            today_target + chrono::Duration::days(1)
        };

        next.signed_duration_since(now)
            .num_milliseconds()
            .max(0)
            .try_into()
            .ok()
    }
}
