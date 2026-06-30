//! 导入命令接口
//!
//! 提供导入/导出相关的 Tauri 命令，处理前端交互。

use crate::application::services::ImportApplicationService;
use crate::domain::models::import::ParsedCurl;
use crate::domain::models::Collection;

#[tauri::command]
pub fn preview_openapi(
    content: String,
    format: String,
    root_name: Option<String>,
) -> Result<Collection, String> {
    ImportApplicationService::preview_openapi(&content, &format, root_name)
}

#[tauri::command]
pub fn import_openapi(
    workspace_id: String,
    content: String,
    format: String,
    target_collection_id: Option<String>,
    root_name: Option<String>,
) -> Result<Collection, String> {
    ImportApplicationService::import_openapi(
        &workspace_id,
        &content,
        &format,
        target_collection_id.as_deref(),
        root_name,
    )
}

#[tauri::command]
pub fn export_collection_postman(
    workspace_id: String,
    collection_id: String,
) -> Result<String, String> {
    ImportApplicationService::export_collection_postman(&workspace_id, &collection_id)
}

#[tauri::command]
pub fn export_collection_postman_with_data(collection: Collection) -> Result<String, String> {
    ImportApplicationService::export_collection_postman_with_data(&collection)
}

#[tauri::command]
pub fn parse_curl(curl_command: String) -> Result<ParsedCurl, String> {
    ImportApplicationService::parse_curl(&curl_command)
}

#[tauri::command]
pub fn preview_postman(content: String, root_name: Option<String>) -> Result<Collection, String> {
    ImportApplicationService::preview_postman(&content, root_name)
}

#[tauri::command]
pub fn import_postman(
    workspace_id: String,
    content: String,
    target_collection_id: Option<String>,
    root_name: Option<String>,
) -> Result<Collection, String> {
    ImportApplicationService::import_postman(
        &workspace_id,
        &content,
        target_collection_id.as_deref(),
        root_name,
    )
}
