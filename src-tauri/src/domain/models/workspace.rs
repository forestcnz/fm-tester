use super::AppSettings;
use serde::{Deserialize, Serialize};

/// 工作区（聚合根 Aggregate Root）
///
/// 这是工作区聚合的根实体，管理用户的所有项目数据。
///
/// ## 聚合边界
/// - 一个 Workspace 聚合包含：工作区本身 + 配置
/// - 工作区内的集合、环境、历史等是独立的聚合
///
/// ## 业务规则
/// - 工作区名称不能为空
///
/// ## 生命周期
/// - 创建：通过 `AppConfigDomainService::create_workspace_entity()`
/// - 验证：通过 `Workspace.validate()` 方法
/// - 持久化：通过 `AppConfigRepository` 仓储接口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub last_opened: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_api_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_backup_at: Option<String>,
}

impl Workspace {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("工作区名称不能为空".to_string());
        }
        Ok(())
    }
}

/// 应用配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub settings: AppSettings,
    pub workspaces: Vec<Workspace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_workspace_id: Option<String>,
}

impl AppConfig {
    pub fn validate(&self) -> Result<(), String> {
        self.settings.validate()?;

        for workspace in &self.workspaces {
            workspace.validate()?;
        }

        for (i, ws1) in self.workspaces.iter().enumerate() {
            for ws2 in self.workspaces.iter().skip(i + 1) {
                if ws1.name == ws2.name {
                    return Err(format!("工作区名称 '{}' 重复", ws1.name));
                }
                if ws1.id == ws2.id {
                    return Err(format!("工作区 ID '{}' 重复", ws1.id));
                }
            }
        }

        Ok(())
    }

    pub fn find_workspace_by_id(&self, id: &str) -> Option<&Workspace> {
        self.workspaces.iter().find(|w| w.id == id)
    }

    pub fn find_workspace_by_name(&self, name: &str) -> Option<&Workspace> {
        self.workspaces.iter().find(|w| w.name == name)
    }
}
