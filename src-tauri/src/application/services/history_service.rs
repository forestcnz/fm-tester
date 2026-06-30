//! History 应用服务
//!
//! 协调历史记录的业务操作，通过仓储工厂动态获取仓储。

use crate::domain::models::{FormField, Header, HistoryEntry, HttpResponse};
use crate::domain::services::HistoryDomainService;
use crate::infrastructure::RepositoryFactory;

/// History 应用服务
pub struct HistoryApplicationService;

impl HistoryApplicationService {
    /// 创建历史记录实体（封装Domain服务）
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
        HistoryDomainService::create_history_entry(
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
        )
    }

    /// 创建并保存历史记录（组合创建和保存操作）
    pub fn create_and_save_history(
        workspace_id: &str,
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
        let entry = Self::create_history_entry(
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
        Self::save_entry(workspace_id, &entry)
    }

    /// 获取所有有历史记录的日期列表
    pub fn get_dates(workspace_id: &str) -> Result<Vec<String>, String> {
        let repository = RepositoryFactory::get_history_repository();
        repository.list_dates(workspace_id)
    }

    /// 获取指定日期的历史记录列表
    pub fn get_by_date(workspace_id: &str, date: &str) -> Result<Vec<HistoryEntry>, String> {
        let repository = RepositoryFactory::get_history_repository();
        repository.get_by_date(workspace_id, date)
    }

    /// 获取指定接口的最近历史记录（按 created_at 倒序）
    pub fn get_by_api(
        workspace_id: &str,
        api_id: &str,
        limit: usize,
    ) -> Result<Vec<HistoryEntry>, String> {
        let repository = RepositoryFactory::get_history_repository();
        repository.get_by_api(workspace_id, api_id, limit)
    }

    /// 获取单个历史记录详情
    pub fn get_entry(
        workspace_id: &str,
        date: &str,
        id: &str,
    ) -> Result<Option<HistoryEntry>, String> {
        let repository = RepositoryFactory::get_history_repository();
        repository.get_entry(workspace_id, date, id)
    }

    /// 保存历史记录
    pub fn save_entry(workspace_id: &str, entry: &HistoryEntry) -> Result<(), String> {
        let repository = RepositoryFactory::get_history_repository();
        repository.save_entry(workspace_id, entry)
    }

    /// 删除单个历史记录
    pub fn delete_entry(workspace_id: &str, date: &str, id: &str) -> Result<(), String> {
        let repository = RepositoryFactory::get_history_repository();
        repository.delete_entry(workspace_id, date, id)
    }

    /// 清空指定日期的历史记录
    pub fn clear_by_date(workspace_id: &str, date: &str) -> Result<(), String> {
        let repository = RepositoryFactory::get_history_repository();
        repository.clear_by_date(workspace_id, date)
    }

    /// 清空所有历史记录
    pub fn clear_all(workspace_id: &str) -> Result<(), String> {
        let repository = RepositoryFactory::get_history_repository();
        repository.clear_all(workspace_id)
    }
}
