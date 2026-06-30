//! 应用配置领域服务
//!
//! 合并 Settings + Workspace 领域服务（都操作 AppConfig）
//!
//! ## 设计理由
//! - SettingsDomainService 和 WorkspaceDomainService 都操作 AppConfig
//! - 合并减少冗余，统一配置操作入口

use crate::domain::models::common::generate_id;
use crate::domain::models::{AppConfig, AppSettings, Workspace};
use chrono::Local;

/// 应用配置领域服务
pub struct AppConfigDomainService;

impl AppConfigDomainService {
    // === Settings 相关 ===

    /// 验证设置
    pub fn validate_settings(settings: &AppSettings) -> Result<(), String> {
        settings.validate()
    }

    // === Workspace 相关 ===

    /// 检查工作区名称是否重复
    pub fn check_name_duplicate(config: &AppConfig, name: &str) -> Result<(), String> {
        if config.workspaces.iter().any(|w| w.name == name) {
            return Err(format!("工作区名称 '{}' 已存在", name));
        }
        Ok(())
    }

    /// 生成工作区 ID
    pub fn generate_workspace_id() -> String {
        generate_id("ws")
    }

    /// 验证工作区切换
    pub fn validate_switch(config: &AppConfig, workspace_id: &str) -> Result<(), String> {
        if !config.workspaces.iter().any(|w| w.id == workspace_id) {
            return Err("工作区不存在".to_string());
        }
        Ok(())
    }

    /// 验证工作区删除
    pub fn validate_delete(config: &AppConfig, workspace_id: &str) -> Result<(), String> {
        if !config.workspaces.iter().any(|w| w.id == workspace_id) {
            return Err("工作区不存在".to_string());
        }
        Ok(())
    }

    /// 创建工作区对象
    pub fn create_workspace_entity(name: String, description: String) -> Workspace {
        let id = Self::generate_workspace_id();
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        Workspace {
            id,
            name,
            description,
            created_at: now.clone(),
            last_opened: now,
            last_api_id: None,
            last_backup_at: None,
        }
    }
}
