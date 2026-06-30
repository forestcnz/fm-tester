//! Response 命令
//!
//! 提供响应快照相关的 Tauri 命令，调用应用服务进行业务处理。

use crate::application::services::ResponseApplicationService;
use crate::domain::models::{SavedResponse, SavedResponseIndexEntry};
use tauri::command;

/// 保存响应快照（保存 MD 文档）
#[command]
pub fn save_response(
    workspace_id: String,
    name: String,
    api_id: Option<String>,
    doc_content: String,
) -> Result<SavedResponse, String> {
    // 使用 Application 服务创建实体
    let saved_response =
        ResponseApplicationService::create_saved_response(name, api_id, doc_content);

    // 创建索引条目
    let index_entry = ResponseApplicationService::create_index_entry(&saved_response);

    // 保存
    ResponseApplicationService::save(&workspace_id, &saved_response, &index_entry)?;

    Ok(saved_response)
}

/// 获取响应索引列表
#[command]
pub fn get_saved_responses(workspace_id: String) -> Result<Vec<SavedResponseIndexEntry>, String> {
    ResponseApplicationService::get_all(&workspace_id)
}

/// 获取单个响应详情
#[command]
pub fn get_saved_response(workspace_id: String, id: String) -> Result<SavedResponse, String> {
    ResponseApplicationService::get(&workspace_id, &id)?.ok_or_else(|| "响应不存在".to_string())
}

/// 删除响应快照
#[command]
pub fn delete_saved_response(workspace_id: String, id: String) -> Result<(), String> {
    ResponseApplicationService::delete(&workspace_id, &id)
}

/// 获取指定 API 的响应列表
#[command]
pub fn get_api_saved_responses(
    workspace_id: String,
    api_id: String,
) -> Result<Vec<SavedResponseIndexEntry>, String> {
    ResponseApplicationService::get_by_api(&workspace_id, &api_id)
}
