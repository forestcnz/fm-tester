//! History 领域服务
//!
//! 提供历史记录相关的纯业务逻辑（ID生成、实体创建）。

use crate::domain::models::common::generate_id;
use crate::domain::models::{FormField, Header, HistoryEntry, HttpResponse};
use chrono::Local;

/// History 领域服务
pub struct HistoryDomainService;

impl HistoryDomainService {
    /// 生成新的历史记录ID
    pub fn generate_id() -> String {
        generate_id("hist")
    }

    /// 生成当前时间戳
    pub fn generate_timestamp() -> String {
        Local::now().to_rfc3339()
    }

    /// 创建 HistoryEntry 实体
    pub fn create_history_entry(
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
    ) -> HistoryEntry {
        let id = Self::generate_id();
        let created_at = Self::generate_timestamp();

        HistoryEntry {
            id,
            method,
            url,
            resolved_url,
            headers,
            body,
            body_type,
            form_fields,
            status: response.status,
            status_text: response.status_text.clone(),
            response_headers: response.headers.clone(),
            response_body: response.body.clone(),
            time: response.time,
            size: response.size,
            created_at,
            api_id,
            api_name,
        }
    }
}
