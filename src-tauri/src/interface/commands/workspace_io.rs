//! 工作区导入导出命令接口

use crate::application::services::WorkspaceIOService;
use crate::domain::models::{Workspace, WorkspaceImportPreview};

#[tauri::command]
pub fn export_workspace(workspace_id: String) -> Result<String, String> {
    let service = WorkspaceIOService::new();
    service.export_workspace(&workspace_id)
}

#[tauri::command]
pub fn preview_workspace_import(content: String) -> Result<WorkspaceImportPreview, String> {
    WorkspaceIOService::preview_import(&content)
}

#[tauri::command]
pub fn import_workspace(content: String, new_name: Option<String>) -> Result<Workspace, String> {
    let service = WorkspaceIOService::new();
    service.import_workspace(&content, new_name)
}
