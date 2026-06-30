//! History 命令
//!
//! 提供历史记录相关的 Tauri 命令，调用应用服务进行业务处理。

use crate::application::services::HistoryApplicationService;
use crate::domain::models::{FormField, Header, HistoryEntry, HttpResponse};
use tauri::command;

/// 获取所有历史记录日期列表
#[command]
pub fn get_history_dates(workspace_id: String) -> Result<Vec<String>, String> {
    HistoryApplicationService::get_dates(&workspace_id)
}

/// 获取指定日期的历史记录列表
#[command]
pub fn get_history_by_date(
    workspace_id: String,
    date: String,
) -> Result<Vec<HistoryEntry>, String> {
    HistoryApplicationService::get_by_date(&workspace_id, &date)
}

/// 获取单个历史记录详情
#[command]
pub fn get_history_entry(
    workspace_id: String,
    date: String,
    id: String,
) -> Result<HistoryEntry, String> {
    HistoryApplicationService::get_entry(&workspace_id, &date, &id)?
        .ok_or_else(|| "历史记录不存在".to_string())
}

/// 记录请求历史（在 send_http_request 中调用）
pub fn record_history(
    workspace_id: String,
    method: String,
    url: String,
    resolved_url: String,
    headers: Vec<Header>,
    body: Option<String>,
    body_type: Option<String>,
    form_fields: Option<Vec<FormField>>,
    response: &HttpResponse,
    api_id: Option<String>,
    api_name: Option<String>,
) -> Result<(), String> {
    // 使用 Application 服务创建实体
    let entry = HistoryApplicationService::create_history_entry(
        method,
        url,
        resolved_url,
        headers,
        body,
        body_type,
        form_fields,
        response,
        api_id,
        api_name,
    );

    HistoryApplicationService::save_entry(&workspace_id, &entry)
}

/// 删除单条历史记录
#[command]
pub fn delete_history_entry(workspace_id: String, date: String, id: String) -> Result<(), String> {
    HistoryApplicationService::delete_entry(&workspace_id, &date, &id)
}

/// 清空指定日期的历史记录
#[command]
pub fn clear_history_by_date(workspace_id: String, date: String) -> Result<(), String> {
    HistoryApplicationService::clear_by_date(&workspace_id, &date)
}

/// 清空所有历史记录
#[command]
pub fn clear_all_history(workspace_id: String) -> Result<(), String> {
    HistoryApplicationService::clear_all(&workspace_id)
}
