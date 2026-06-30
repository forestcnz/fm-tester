//! 工作区数据应用服务
//!
//! environment / memory / cookie 的业务编排。
//!
//! ## 设计理由
//! - 三者的持久化操作在 WorkspaceDataRepository 中独立读写
//! - 应用服务合并，保持架构一致性
//! - 减少服务数量，简化代码结构

use crate::domain::models::{Cookie, Environment, EnvironmentsConfig, ReplaceResult};
use crate::domain::services::WorkspaceDataDomainService;
use crate::infrastructure::RepositoryFactory;
use std::collections::HashMap;

/// 工作区数据应用服务
///
/// 无状态服务，每次调用时通过工厂获取仓储
pub struct WorkspaceDataApplicationService;

impl WorkspaceDataApplicationService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WorkspaceDataApplicationService {
    fn default() -> Self {
        Self::new()
    }
}

// === Environment 相关 ===

impl WorkspaceDataApplicationService {
    /// 创建环境（纯验证，不持久化）
    pub fn create_environment(name: String) -> Result<Environment, String> {
        WorkspaceDataDomainService::validate_variable_key(&name)?;
        let env = WorkspaceDataDomainService::create_environment_entity(name);
        WorkspaceDataDomainService::validate_environment(&env)?;
        Ok(env)
    }

    /// 读取环境配置
    pub fn read_environments(workspace_id: &str) -> Result<EnvironmentsConfig, String> {
        let repository = RepositoryFactory::get_workspace_data_repository();
        repository.read_environments(workspace_id)
    }

    /// 写入环境配置
    pub fn write_environments(
        workspace_id: &str,
        config: &EnvironmentsConfig,
    ) -> Result<(), String> {
        let repository = RepositoryFactory::get_workspace_data_repository();
        repository.write_environments(workspace_id, config)
    }

    /// 获取当前激活环境的变量映射
    pub fn get_active_variables_map(config: &EnvironmentsConfig) -> HashMap<String, String> {
        WorkspaceDataDomainService::get_active_variables_map(config)
    }

    /// 验证变量键名
    pub fn validate_variable_key(name: &str) -> Result<(), String> {
        WorkspaceDataDomainService::validate_variable_key(name)
    }

    /// 替换字符串中的变量
    pub fn replace_variables(text: &str, variables: &HashMap<String, String>) -> ReplaceResult {
        WorkspaceDataDomainService::replace_variables(text, variables)
    }
}

// === Memory 相关 ===

impl WorkspaceDataApplicationService {
    /// 获取展开的集合ID列表
    pub fn get_expanded_collections(workspace_id: &str) -> Result<Vec<String>, String> {
        let repo = RepositoryFactory::get_workspace_data_repository();
        let config = repo.read_memory(workspace_id)?;
        Ok(config.expanded_ids)
    }

    /// 保存展开的集合ID列表
    pub fn save_expanded_collections(
        workspace_id: &str,
        expanded_ids: Vec<String>,
    ) -> Result<(), String> {
        WorkspaceDataDomainService::validate_expanded_ids(&expanded_ids)?;

        let repo = RepositoryFactory::get_workspace_data_repository();
        let mut config = repo.read_memory(workspace_id)?;
        config.expanded_ids = expanded_ids;

        WorkspaceDataDomainService::validate_memory_config(&config)?;

        repo.write_memory(workspace_id, &config)
    }

    /// 获取打开的标签页数据
    pub fn get_open_tabs(
        workspace_id: &str,
    ) -> Result<(Vec<String>, Vec<String>, usize, HashMap<String, String>), String> {
        let repo = RepositoryFactory::get_workspace_data_repository();
        let config = repo.read_memory(workspace_id)?;
        Ok((
            config.open_tabs,
            config.open_tab_types,
            config.active_tab_index,
            config.request_tabs,
        ))
    }

    /// 保存打开的标签页数据
    pub fn save_open_tabs(
        workspace_id: &str,
        open_tabs: Vec<String>,
        open_tab_types: Vec<String>,
        active_tab_index: usize,
        request_tabs: HashMap<String, String>,
    ) -> Result<(), String> {
        WorkspaceDataDomainService::validate_open_tabs(&open_tabs)?;
        WorkspaceDataDomainService::validate_active_tab_index(open_tabs.len(), active_tab_index)?;

        let repo = RepositoryFactory::get_workspace_data_repository();
        let mut config = repo.read_memory(workspace_id)?;
        config.open_tabs = open_tabs;
        config.open_tab_types = open_tab_types;
        config.active_tab_index = active_tab_index;
        config.request_tabs = request_tabs;

        WorkspaceDataDomainService::validate_memory_config(&config)?;

        repo.write_memory(workspace_id, &config)
    }
}

// === Cookie 相关 ===

impl WorkspaceDataApplicationService {
    /// 获取所有 Cookie
    pub fn get_cookies(workspace_id: &str) -> Result<Vec<Cookie>, String> {
        let repo = RepositoryFactory::get_workspace_data_repository();
        repo.get_all_cookies(workspace_id)
    }

    /// 添加或更新 Cookie
    ///
    /// 如果存在相同 name + domain 的 Cookie，则更新；否则添加新 Cookie
    pub fn add_cookie(workspace_id: &str, cookie: &Cookie) -> Result<(), String> {
        WorkspaceDataDomainService::validate_cookie(cookie)?;
        let repo = RepositoryFactory::get_workspace_data_repository();
        repo.add_or_update_cookie(workspace_id, cookie)
    }

    /// 删除指定 Cookie
    pub fn delete_cookie(workspace_id: &str, name: &str, domain: &str) -> Result<(), String> {
        let repo = RepositoryFactory::get_workspace_data_repository();
        repo.delete_cookie(workspace_id, name, domain)
    }

    /// 清除所有 Cookie
    pub fn clear_cookies(workspace_id: &str) -> Result<(), String> {
        let repo = RepositoryFactory::get_workspace_data_repository();
        repo.clear_cookies(workspace_id)
    }
}

// ==================== 模块级公共函数（向后兼容）====================

/// 替换变量（便捷函数）
pub fn replace_variables(text: &str, variables: &HashMap<String, String>) -> ReplaceResult {
    WorkspaceDataApplicationService::replace_variables(text, variables)
}
