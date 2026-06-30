//! 压力测试命令接口
//!
//! 提供压力测试相关的 Tauri 命令，调用应用服务进行业务处理。

use crate::application::services::ScriptApplicationService;
use crate::application::services::StressApplicationService;
use crate::domain::models::{
    ScriptInfo, StressParamsConfig, StressTestConfig, StressTestProgress, StressTestResult,
    StressTestResultIndexEntry,
};
use crate::domain::models::{ScriptKind, ScriptTargetType};
use crate::domain::services::stress::runner_domain::StressTestRunnerCore;
use crate::domain::services::ScriptExecutionDomainService;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref RUNNING_TESTS: Mutex<HashMap<String, Arc<StressTestRunnerCore>>> =
        Mutex::new(HashMap::new());
}

/// 加载后置脚本链
fn load_post_scripts(
    workspace_id: &str,
    api_id: Option<&str>,
    environment_id: Option<&str>,
    ancestor_collections: &[(String, String)],
) -> Vec<ScriptInfo> {
    let mut scripts = Vec::new();

    // 1. 工作区脚本
    if let Ok(content) = ScriptApplicationService::get(
        workspace_id,
        ScriptTargetType::Workspace,
        None,
        ScriptKind::Post,
    ) {
        if !ScriptExecutionDomainService::is_empty_script(&content) {
            scripts.push(ScriptInfo {
                source: "workspace".to_string(),
                content,
                source_type: ScriptTargetType::Workspace,
                target_id: None,
            });
        }
    }

    // 2. 环境脚本
    if let Some(env_id) = environment_id {
        if let Ok(content) = ScriptApplicationService::get(
            workspace_id,
            ScriptTargetType::Environment,
            Some(env_id.to_string()),
            ScriptKind::Post,
        ) {
            if !ScriptExecutionDomainService::is_empty_script(&content) {
                scripts.push(ScriptInfo {
                    source: "environment".to_string(),
                    content,
                    source_type: ScriptTargetType::Environment,
                    target_id: Some(env_id.to_string()),
                });
            }
        }
    }

    // 3. 集合脚本（按层级顺序）
    for (coll_id, coll_name) in ancestor_collections {
        if let Ok(content) = ScriptApplicationService::get(
            workspace_id,
            ScriptTargetType::Collection,
            Some(coll_id.clone()),
            ScriptKind::Post,
        ) {
            if !ScriptExecutionDomainService::is_empty_script(&content) {
                scripts.push(ScriptInfo {
                    source: format!("collection:{}", coll_name),
                    content,
                    source_type: ScriptTargetType::Collection,
                    target_id: Some(coll_id.clone()),
                });
            }
        }
    }

    // 4. 接口脚本
    if let Some(api_id) = api_id {
        if let Ok(content) = ScriptApplicationService::get(
            workspace_id,
            ScriptTargetType::Api,
            Some(api_id.to_string()),
            ScriptKind::Post,
        ) {
            if !ScriptExecutionDomainService::is_empty_script(&content) {
                scripts.push(ScriptInfo {
                    source: "api".to_string(),
                    content,
                    source_type: ScriptTargetType::Api,
                    target_id: Some(api_id.to_string()),
                });
            }
        }
    }

    // 后置脚本反向执行
    scripts.reverse();

    scripts
}

/// 启动压测
#[tauri::command]
pub async fn start_stress_test(
    app: AppHandle,
    workspace_id: String,
    config: StressTestConfig,
) -> Result<String, String> {
    let test_id = config.id.clone();

    // 通过 Application 服务获取变量和 Cookies
    let (variables, cookies) = StressApplicationService::get_variables_and_cookies(&workspace_id)?;

    // 加载后置脚本链
    let ancestor_collections: Vec<(String, String)> = config
        .ancestor_collections
        .as_ref()
        .map(|acs| {
            acs.iter()
                .map(|ac| (ac.id.clone(), ac.name.clone()))
                .collect()
        })
        .unwrap_or_default();

    let post_scripts = load_post_scripts(
        &workspace_id,
        config.api_id.as_deref(),
        config.environment_id.as_deref(),
        &ancestor_collections,
    );

    // 使用Application服务创建运行器
    let runner = StressApplicationService::create_runner(config, variables, cookies, post_scripts);

    // 注册运行中的测试
    {
        let mut tests = RUNNING_TESTS.lock().map_err(|e| e.to_string())?;
        tests.insert(test_id.clone(), runner.clone());
    }

    // 克隆必要数据用于异步任务
    let test_id_clone = test_id.clone();
    let workspace_id_clone = workspace_id.clone();
    let runner_clone = runner.clone();
    let app_clone = app.clone();

    // 在后台运行压测
    tokio::spawn(async move {
        let result = run_stress_test(runner_clone.clone(), app_clone.clone()).await;

        // 用户手动停止时，由 stop_stress_test 负责保存，这里跳过
        let user_stopped = runner_clone.stop_signal.load(Ordering::SeqCst);
        if !user_stopped {
            // 自然完成：保存结果
            if let Err(e) = StressApplicationService::save_result(&workspace_id_clone, &result) {
                eprintln!("保存压测结果失败: {}", e);
            }
            // 发送完成事件
            let _ = app_clone.emit("stress-test-complete", result);
        }

        // 从运行列表移除（可能已被 stop_stress_test 移除，这里再检查一次）
        if let Ok(mut tests) = RUNNING_TESTS.lock() {
            tests.remove(&test_id_clone);
        }
    });

    Ok(test_id)
}

/// 运行压测（内部函数）
async fn run_stress_test(runner: Arc<StressTestRunnerCore>, app: AppHandle) -> StressTestResult {
    use std::time::{Duration, Instant};
    use tokio::task::JoinSet;

    let max_concurrent = runner.config.concurrent.min(999);
    let total_requests = runner.config.total_requests;
    let duration_seconds = runner.config.duration_seconds;
    let ramp_up_seconds = runner.config.ramp_up_seconds;

    let mut join_set: JoinSet<()> = JoinSet::new();
    let mut last_progress_time = Instant::now();
    let mut ramp_up_complete = false;
    let mut last_history_second: u32 = 0;

    // 主循环
    loop {
        // 检查是否应该停止发起新请求
        let spawned = runner.stats.spawned.load(Ordering::SeqCst);
        let elapsed_secs = runner.start_time.elapsed().as_secs();
        let user_stopped = runner.stop_signal.load(Ordering::SeqCst);

        let should_stop_spawning = user_stopped
            || if let Some(total) = total_requests {
                spawned >= total
            } else if let Some(duration) = duration_seconds {
                elapsed_secs >= duration as u64
            } else {
                false
            };

        // 如果不再发起新请求，且所有任务都完成了，退出
        if should_stop_spawning && join_set.is_empty() {
            break;
        }

        // 预热阶段 - 逐步增加并发
        let current_max_concurrent = if ramp_up_seconds > 0 && !ramp_up_complete {
            if elapsed_secs < ramp_up_seconds as u64 {
                let ratio = elapsed_secs as f64 / ramp_up_seconds as f64;
                ((max_concurrent as f64 * ratio).max(1.0) as u32).min(max_concurrent)
            } else {
                ramp_up_complete = true;
                max_concurrent
            }
        } else {
            max_concurrent
        };

        // 如果可以继续发起请求，填充任务池
        if !should_stop_spawning {
            while join_set.len() < current_max_concurrent as usize {
                // 先递增 spawned 计数器，如果是总请求数模式且已超则回退并退出
                if let Some(total) = total_requests {
                    let current = runner.stats.spawned.fetch_add(1, Ordering::SeqCst);
                    if current >= total {
                        runner.stats.spawned.fetch_sub(1, Ordering::SeqCst);
                        break;
                    }
                } else {
                    // 持续时间模式：直接递增计数器
                    runner.stats.spawned.fetch_add(1, Ordering::SeqCst);
                }

                // 发起请求
                let runner_clone = runner.clone();
                join_set.spawn(async move {
                    let (success, status, error, elapsed_ms) =
                        runner_clone.send_single_request().await;

                    // 更新统计数据
                    runner_clone.stats.completed.fetch_add(1, Ordering::SeqCst);
                    if success {
                        runner_clone.stats.successful.fetch_add(1, Ordering::SeqCst);
                    } else {
                        runner_clone.stats.failed.fetch_add(1, Ordering::SeqCst);
                    }

                    // 记录响应时间
                    if elapsed_ms > 0 {
                        let mut times = runner_clone.stats.times.lock().await;
                        times.push(elapsed_ms);
                    }

                    // 记录状态码分布
                    if let Some(code) = status {
                        let mut status_codes = runner_clone.stats.status_codes.lock().await;
                        // Convert u16 status code to String for TOML compatibility
                        *status_codes.entry(code.to_string()).or_insert(0) += 1;
                    }

                    // 记录错误分布
                    if let Some(err) = &error {
                        let mut errors = runner_clone.stats.errors.lock().await;
                        *errors.entry(err.clone()).or_insert(0) += 1;

                        // 记录失败请求详情（保留最近100条）
                        let mut failed_details =
                            runner_clone.stats.failed_request_details.lock().await;
                        if failed_details.len() < 100 {
                            use crate::domain::models::FailedRequest;
                            let time = chrono::Local::now().format("%H:%M:%S").to_string();
                            failed_details.push(FailedRequest {
                                time,
                                error: error.clone().unwrap(),
                                status,
                                elapsed_ms,
                            });
                        }
                    }
                });
            }
        }

        // 等待任务完成
        if should_stop_spawning {
            // 已达到停止条件，阻塞等待所有剩余任务完成
            while !join_set.is_empty() {
                join_set.join_next().await;
            }
        } else {
            // 正常运行中，等待部分任务完成以保持并发控制
            while join_set.len() >= current_max_concurrent as usize {
                join_set.join_next().await;
            }
        }

        // 每秒发送一次进度更新
        if last_progress_time.elapsed() >= Duration::from_secs(1) {
            // 更新当前允许的最大并发数（预热阶段会逐步增加）
            runner
                .stats
                .current_running
                .store(current_max_concurrent, Ordering::SeqCst);

            // 记录历史数据点（每秒记录一次）
            let current_second = elapsed_secs as u32;
            if current_second > last_history_second {
                runner.record_history(current_second).await;
                last_history_second = current_second;
            }
            let progress = runner.get_progress().await;
            let _ = app.emit("stress-test-progress", progress);
            last_progress_time = Instant::now();
        }

        // 短暂休眠避免空转
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // 等待所有剩余任务完成
    while join_set.try_join_next().is_some() {}

    // 发送最终进度（停止状态）
    let elapsed_seconds = runner.start_time.elapsed().as_secs() as u32;
    runner.record_history(elapsed_seconds).await;
    let progress = runner.get_progress().await;
    let _ = app.emit("stress-test-progress", progress);

    // 计算最终结果
    runner.calculate_result().await
}

/// 获取压测进度
#[tauri::command]
pub fn get_stress_test_progress(id: String) -> Result<StressTestProgress, String> {
    let tests = RUNNING_TESTS.lock().map_err(|e| e.to_string())?;

    if let Some(runner) = tests.get(&id) {
        let completed = runner.stats.completed.load(Ordering::SeqCst);
        let successful = runner.stats.successful.load(Ordering::SeqCst);
        let failed = runner.stats.failed.load(Ordering::SeqCst);
        let elapsed_seconds = runner.start_time.elapsed().as_secs() as u32;

        let avg_time = if let Ok(times) = runner.stats.times.try_lock() {
            if times.is_empty() {
                0.0
            } else {
                times.iter().sum::<u64>() as f64 / times.len() as f64
            }
        } else {
            0.0
        };

        let current_qps = if elapsed_seconds > 0 {
            (completed as f64) / (elapsed_seconds as f64)
        } else {
            0.0
        };

        let history = if let Ok(h) = runner.stats.history.try_lock() {
            h.clone()
        } else {
            vec![]
        };

        Ok(StressTestProgress {
            id: id.clone(),
            elapsed_seconds,
            completed_requests: completed,
            current_qps,
            current_avg_time_ms: avg_time,
            successful_requests: successful,
            failed_requests: failed,
            current_concurrent: runner.stats.current_running.load(Ordering::SeqCst),
            is_running: !runner.stop_signal.load(Ordering::SeqCst),
            history,
        })
    } else {
        Err(format!("测试 {} 未找到或已结束", id))
    }
}

/// 停止压测
#[tauri::command]
pub async fn stop_stress_test(
    app: AppHandle,
    id: String,
    workspace_id: String,
) -> Result<StressTestResult, String> {
    let runner = {
        let tests = RUNNING_TESTS.lock().map_err(|e| e.to_string())?;
        tests
            .get(&id)
            .cloned()
            .ok_or_else(|| format!("测试 {} 未找到", id))?
    };

    runner.stop();

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let result = runner.calculate_result().await;

    StressApplicationService::save_result(&workspace_id, &result)?;

    let _ = app.emit("stress-test-complete", result.clone());

    if let Ok(mut tests) = RUNNING_TESTS.lock() {
        tests.remove(&id);
    }

    Ok(result)
}

/// 获取接口的压测记录列表
#[tauri::command]
pub fn get_api_stress_test_results(
    workspace_id: String,
    api_id: String,
) -> Result<Vec<StressTestResultIndexEntry>, String> {
    StressApplicationService::get_api_results(&workspace_id, &api_id)
}

/// 获取单个压测结果
#[tauri::command]
pub fn get_stress_test_result(
    workspace_id: String,
    api_id: String,
    id: String,
) -> Result<StressTestResult, String> {
    StressApplicationService::get_result(&workspace_id, &api_id, &id)?
        .ok_or_else(|| format!("结果 {} 未找到", id))
}

/// 删除压测结果
#[tauri::command]
pub fn delete_stress_test_result(
    workspace_id: String,
    api_id: String,
    id: String,
) -> Result<(), String> {
    StressApplicationService::delete_result(&workspace_id, &api_id, &id)
}

/// 获取压测参数配置
#[tauri::command]
pub fn get_stress_params(
    workspace_id: String,
    api_id: String,
) -> Result<StressParamsConfig, String> {
    StressApplicationService::get_params(&workspace_id, &api_id)
}

/// 保存压测参数配置
#[tauri::command]
pub fn save_stress_params(
    workspace_id: String,
    api_id: String,
    config: StressParamsConfig,
) -> Result<(), String> {
    StressApplicationService::save_params(&workspace_id, &api_id, &config)
}

/// 清理已停止的压测任务（防止 HashMap 无限增长）
///
/// 自动清理 stop_signal 为 true 的任务
pub fn cleanup_stopped_stress_tests() {
    if let Ok(mut tests) = RUNNING_TESTS.lock() {
        tests.retain(|_id, runner| !runner.stop_signal.load(Ordering::SeqCst));
    }
}

/// 获取当前运行中的压测任务数量（用于监控）
pub fn get_running_test_count() -> usize {
    if let Ok(tests) = RUNNING_TESTS.lock() {
        tests.len()
    } else {
        0
    }
}
