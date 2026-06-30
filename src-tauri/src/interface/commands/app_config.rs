//! 应用配置 Tauri 命令
//!
//! 合并 Settings + Workspace 命令（都操作 AppConfig）

use crate::application::services::{
    read_collections, AppConfigApplicationService, CollectionApplicationService,
};
use crate::domain::models::{
    AppSettings, AppearanceSettings, BehaviorSettings, Collection, Header, Workspace,
};
use tauri::{AppHandle, Emitter};

// === Settings 命令 ===

/// 获取全局设置
#[tauri::command]
pub fn get_settings() -> Result<AppSettings, String> {
    let service = AppConfigApplicationService::default();
    service.get_settings()
}

/// 更新全局设置
#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    timeout: u64,
    language: Option<String>,
    ai_api_endpoint: Option<String>,
    ai_api_key: Option<String>,
    ai_model: Option<String>,
    ai_custom_headers: Option<Vec<Header>>,
    ai_timeout: Option<u64>,
    appearance: Option<AppearanceSettings>,
    behavior: Option<BehaviorSettings>,
) -> Result<AppSettings, String> {
    let service = AppConfigApplicationService::default();
    let settings = service.update_settings(
        timeout,
        language,
        ai_api_endpoint,
        ai_api_key,
        ai_model,
        ai_custom_headers,
        ai_timeout,
        appearance,
        behavior,
    )?;

    // 发送设置更新事件
    app.emit("settings-updated", &settings).ok();

    Ok(settings)
}

// === Workspace 命令 ===

/// 获取所有工作区
#[tauri::command]
pub fn get_workspaces() -> Result<Vec<Workspace>, String> {
    let service = AppConfigApplicationService::default();
    service.get_all_workspaces()
}

/// 获取最近打开的工作区
#[tauri::command]
pub fn get_last_workspace() -> Result<Option<Workspace>, String> {
    let service = AppConfigApplicationService::default();
    service.get_last_workspace()
}

/// 创建新工作区
#[tauri::command]
pub fn create_workspace(name: String, description: String) -> Result<Workspace, String> {
    let service = AppConfigApplicationService::default();
    service.create_workspace(name, description)
}

/// 切换工作区
#[tauri::command]
pub fn switch_workspace(id: String) -> Result<Workspace, String> {
    let service = AppConfigApplicationService::default();
    service.switch_workspace(id)
}

/// 删除工作区
#[tauri::command]
pub fn delete_workspace(id: String) -> Result<(), String> {
    let service = AppConfigApplicationService::default();
    service.delete_workspace(id)
}

/// 更新工作区信息
#[tauri::command]
pub fn update_workspace(id: String, name: String, description: String) -> Result<(), String> {
    let service = AppConfigApplicationService::default();
    service.update_workspace(id, name, description)
}

/// 设置最后打开的接口
#[tauri::command]
pub fn set_last_api(workspace_id: String, api_id: String) -> Result<(), String> {
    let service = AppConfigApplicationService::default();
    service.set_last_api(workspace_id, api_id)
}

/// 获取最后打开的接口
#[tauri::command]
pub fn get_last_api(workspace_id: String) -> Result<Option<Collection>, String> {
    let service = AppConfigApplicationService::default();

    let api_id = service.get_last_api_id(&workspace_id)?;

    if let Some(id) = api_id {
        let collections = read_collections(&workspace_id)?;
        if let Some(api) = CollectionApplicationService::find_api(&collections, &id) {
            return Ok(Some(api));
        }
    }

    Ok(None)
}

/// 设置最后打开的工作区
#[tauri::command]
pub fn set_last_workspace(workspace_id: String) -> Result<(), String> {
    let service = AppConfigApplicationService::default();
    service.set_last_workspace(workspace_id)
}

/// 工作区排序
#[tauri::command]
pub fn reorder_workspaces(workspace_id: String, new_index: usize) -> Result<(), String> {
    let service = AppConfigApplicationService::default();
    service.reorder_workspaces(workspace_id, new_index)
}
