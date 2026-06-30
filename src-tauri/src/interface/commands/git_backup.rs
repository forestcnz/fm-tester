//! Git 工作区备份 Tauri 命令

use crate::application::services::AppConfigApplicationService;
use crate::domain::models::{GitBackupFile, GitBackupSettingsView, Workspace};
use crate::infrastructure::git_backup;
use tauri::AppHandle;

/// 获取 Git 备份配置（密码脱敏）
#[tauri::command]
pub fn get_git_backup_settings() -> Result<GitBackupSettingsView, String> {
    let service = AppConfigApplicationService::default();
    service.get_git_backup_settings()
}

/// 更新 Git 备份配置
///
/// password 三态语义：None=保持原值、空串=清空、非空串=加密保存
#[tauri::command]
pub fn update_git_backup_settings(
    repo_url: Option<String>,
    branch: Option<String>,
    username: Option<String>,
    password: Option<String>,
) -> Result<GitBackupSettingsView, String> {
    let service = AppConfigApplicationService::default();
    service.update_git_backup_settings(repo_url, branch, username, password)
}

/// 更新自动备份配置（开关 + 每日备份时刻 + 目标工作区），并重启后台任务
#[tauri::command]
pub fn update_auto_backup_settings(
    app: AppHandle,
    enabled: bool,
    time: String,
    workspace_ids: Vec<String>,
) -> Result<(), String> {
    let service = AppConfigApplicationService::default();
    service.update_auto_backup_settings(enabled, time, workspace_ids)?;
    crate::infrastructure::auto_backup::AutoBackupScheduler::start(app);
    Ok(())
}

/// 测试 Git 备份设备连接，返回远程分支列表
#[tauri::command]
pub async fn test_git_connection() -> Result<Vec<String>, String> {
    // git2 为阻塞操作，在独立线程执行避免阻塞 runtime
    tokio::task::spawn_blocking(git_backup::test_connection)
        .await
        .map_err(|e| format!("测试连接任务失败: {}", e))?
}

/// 备份工作区（导出 JSON 并推送至 Git）
#[tauri::command]
pub async fn backup_workspace(workspace_id: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || git_backup::backup_workspace(&workspace_id))
        .await
        .map_err(|e| format!("备份任务失败: {}", e))?
}

/// 列举备份仓库中的所有备份文件
#[tauri::command]
pub async fn list_workspace_backups() -> Result<Vec<GitBackupFile>, String> {
    tokio::task::spawn_blocking(git_backup::list_backups)
        .await
        .map_err(|e| format!("列举备份任务失败: {}", e))?
}

/// 从备份恢复为新工作区
#[tauri::command]
pub async fn restore_workspace_from_backup(
    workspace_name: String,
    file_name: String,
    new_name: Option<String>,
) -> Result<Workspace, String> {
    tokio::task::spawn_blocking(move || {
        git_backup::restore_from_backup(&workspace_name, &file_name, new_name)
    })
    .await
    .map_err(|e| format!("恢复任务失败: {}", e))?
}

/// 从备份覆盖恢复到指定工作区（保留目标工作区 id 与名称，替换全部数据）
#[tauri::command]
pub async fn restore_into_workspace(
    target_workspace_id: String,
    workspace_name: String,
    file_name: String,
) -> Result<Workspace, String> {
    tokio::task::spawn_blocking(move || {
        git_backup::restore_into_workspace(&target_workspace_id, &workspace_name, &file_name)
    })
    .await
    .map_err(|e| format!("恢复任务失败: {}", e))?
}

/// 删除指定备份文件
#[tauri::command]
pub async fn delete_backup(workspace_name: String, file_name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || git_backup::delete_backup(&workspace_name, &file_name))
        .await
        .map_err(|e| format!("删除备份任务失败: {}", e))?
}
