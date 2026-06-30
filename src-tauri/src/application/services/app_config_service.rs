//! 应用配置应用服务
//!
//! 合并 Settings + Workspace 应用服务（都操作 AppConfig）

use crate::domain::models::{
    AiSettings, AppSettings, AppearanceSettings, BehaviorSettings, GitBackupSettingsView, Header,
    Workspace,
};
use crate::domain::repositories::AppConfigRepository;
use crate::domain::services::{AppConfigDomainService, EncryptionService};
use crate::infrastructure::{get_encryption_service, RepositoryFactory};

/// 应用配置应用服务
pub struct AppConfigApplicationService {
    repository: Box<dyn AppConfigRepository>,
    encryption: Box<dyn EncryptionService>,
}

impl AppConfigApplicationService {
    pub fn new(
        repository: Box<dyn AppConfigRepository>,
        encryption: Box<dyn EncryptionService>,
    ) -> Self {
        Self {
            repository,
            encryption,
        }
    }

    pub fn default() -> Self {
        Self::new(
            RepositoryFactory::get_app_config_repository(),
            Box::new(get_encryption_service()),
        )
    }
}

impl Default for AppConfigApplicationService {
    fn default() -> Self {
        Self::default()
    }
}

impl AppConfigApplicationService {
    // === Settings 相关 ===

    /// 获取设置（隐藏 API Key）
    pub fn get_settings(&self) -> Result<AppSettings, String> {
        let config = self.repository.read()?;
        let mut settings = config.settings.clone();

        if settings.ai.encrypted_api_key.is_empty() {
            settings.ai.encrypted_api_key = "".to_string();
        } else if let Ok(decrypted_key) = self.encryption.decrypt(&settings.ai.encrypted_api_key) {
            settings.ai.encrypted_api_key = Self::mask_api_key(&decrypted_key);
        } else {
            settings.ai.encrypted_api_key = "***".to_string();
        }

        // Git 备份密码不随通用设置返回，避免密文泄露（专用命令返回脱敏视图）
        settings.git_backup.encrypted_password = String::new();

        Ok(settings)
    }

    fn mask_api_key(key: &str) -> String {
        let chars: Vec<char> = key.chars().collect();
        let len = chars.len();
        if len <= 8 {
            let prefix_len = len.min(4);
            let mask_len = len - prefix_len;
            let prefix: String = chars[..prefix_len].iter().collect();
            format!("{}{}", prefix, "*".repeat(mask_len))
        } else {
            let prefix_len = 4;
            let suffix_len = 4;
            let mask_len = len - prefix_len - suffix_len;
            let prefix: String = chars[..prefix_len].iter().collect();
            let suffix: String = chars[len - suffix_len..].iter().collect();
            format!("{}{}{}", prefix, "*".repeat(mask_len), suffix)
        }
    }

    /// 获取原始设置（包含加密的 API Key）
    pub fn get_raw_settings(&self) -> Result<AppSettings, String> {
        let config = self.repository.read()?;
        Ok(config.settings)
    }

    /// 获取解密后的 AI 配置
    pub fn get_decrypted_ai_config(&self) -> Result<AiSettings, String> {
        let settings = self.get_raw_settings()?;

        let decrypted_api_key = if settings.ai.encrypted_api_key.is_empty() {
            "".to_string()
        } else {
            self.encryption.decrypt(&settings.ai.encrypted_api_key)?
        };

        Ok(AiSettings {
            api_endpoint: settings.ai.api_endpoint,
            encrypted_api_key: decrypted_api_key,
            model: settings.ai.model,
            custom_headers: settings.ai.custom_headers,
            timeout: settings.ai.timeout,
        })
    }

    /// 更新设置
    pub fn update_settings(
        &self,
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
        let mut config = self.repository.read()?;

        config.settings.request_timeout = timeout;

        if let Some(lang) = language {
            config.settings.language = lang;
        }

        if let Some(endpoint) = ai_api_endpoint {
            config.settings.ai.api_endpoint = endpoint;
        }

        if let Some(key) = ai_api_key {
            if key.is_empty() {
                config.settings.ai.encrypted_api_key = "".to_string();
            } else {
                let encrypted_key = self.encryption.encrypt(&key)?;
                config.settings.ai.encrypted_api_key = encrypted_key;
            }
        }

        if let Some(model) = ai_model {
            config.settings.ai.model = model;
        }

        if let Some(headers) = ai_custom_headers {
            config.settings.ai.custom_headers = headers;
        }

        if let Some(ai_timeout_val) = ai_timeout {
            config.settings.ai.timeout = ai_timeout_val;
        }

        if let Some(appearance) = appearance {
            config.settings.appearance = appearance;
        }
        if let Some(behavior) = behavior {
            config.settings.behavior = behavior;
        }

        AppConfigDomainService::validate_settings(&config.settings)?;

        self.repository.write(&config)?;

        let mut response_settings = config.settings.clone();
        if response_settings.ai.encrypted_api_key.is_empty() {
            response_settings.ai.encrypted_api_key = "".to_string();
        } else if let Ok(decrypted_key) = self
            .encryption
            .decrypt(&response_settings.ai.encrypted_api_key)
        {
            response_settings.ai.encrypted_api_key = Self::mask_api_key(&decrypted_key);
        } else {
            response_settings.ai.encrypted_api_key = "***".to_string();
        }

        Ok(response_settings)
    }

    // === Git 备份相关 ===

    /// 获取 Git 备份配置（密码脱敏，仅返回是否已设置）
    pub fn get_git_backup_settings(&self) -> Result<GitBackupSettingsView, String> {
        let config = self.repository.read()?;
        let g = &config.settings.git_backup;
        Ok(GitBackupSettingsView {
            repo_url: g.repo_url.clone(),
            branch: g.branch.clone(),
            username: g.username.clone(),
            has_password: !g.encrypted_password.is_empty(),
            auto_backup_enabled: g.auto_backup_enabled,
            auto_backup_time: g.auto_backup_time.clone(),
            auto_backup_workspace_ids: g.auto_backup_workspace_ids.clone(),
        })
    }

    /// 更新 Git 备份配置
    ///
    /// password 三态语义：None=保持原值、空串=清空、非空串=加密保存
    pub fn update_git_backup_settings(
        &self,
        repo_url: Option<String>,
        branch: Option<String>,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<GitBackupSettingsView, String> {
        let mut config = self.repository.read()?;
        let g = &mut config.settings.git_backup;

        if let Some(url) = repo_url {
            g.repo_url = url;
        }
        if let Some(b) = branch {
            g.branch = b;
        }
        if let Some(u) = username {
            g.username = u;
        }
        if let Some(p) = password {
            if p.is_empty() {
                g.encrypted_password = String::new();
            } else {
                g.encrypted_password = self.encryption.encrypt(&p)?;
            }
        }

        self.repository.write(&config)?;

        let g2 = &config.settings.git_backup;
        Ok(GitBackupSettingsView {
            repo_url: g2.repo_url.clone(),
            branch: g2.branch.clone(),
            username: g2.username.clone(),
            has_password: !g2.encrypted_password.is_empty(),
            auto_backup_enabled: g2.auto_backup_enabled,
            auto_backup_time: g2.auto_backup_time.clone(),
            auto_backup_workspace_ids: g2.auto_backup_workspace_ids.clone(),
        })
    }

    /// 更新自动备份配置（开关 + 每日备份时刻 + 目标工作区）
    pub fn update_auto_backup_settings(
        &self,
        enabled: bool,
        time: String,
        workspace_ids: Vec<String>,
    ) -> Result<(), String> {
        // 开启时校验时间格式 HH:MM
        if enabled {
            let parts: Vec<&str> = time.split(':').collect();
            let invalid = parts.len() != 2
                || parts[0].parse::<u32>().map(|h| h > 23).unwrap_or(true)
                || parts[1].parse::<u32>().map(|m| m > 59).unwrap_or(true);
            if invalid {
                return Err("备份时间格式无效，需为 HH:MM".to_string());
            }
        }
        let mut config = self.repository.read()?;
        let g = &mut config.settings.git_backup;
        g.auto_backup_enabled = enabled;
        g.auto_backup_time = time;
        // 仅保留去重后的工作区 id（前端负责提供候选列表，此处统一去重防重）
        let mut deduped: Vec<String> = Vec::with_capacity(workspace_ids.len());
        for id in workspace_ids {
            if !deduped.contains(&id) {
                deduped.push(id);
            }
        }
        g.auto_backup_workspace_ids = deduped;
        self.repository.write(&config)?;
        Ok(())
    }

    // === Workspace 相关 ===

    /// 创建工作区
    pub fn create_workspace(&self, name: String, description: String) -> Result<Workspace, String> {
        let config = self.repository.read()?;

        AppConfigDomainService::check_name_duplicate(&config, &name)?;

        let workspace = AppConfigDomainService::create_workspace_entity(name, description);

        let mut config = self.repository.read()?;
        config.workspaces.push(workspace.clone());
        config.last_workspace_id = Some(workspace.id.clone());

        self.repository.write(&config)?;

        Ok(workspace)
    }

    /// 获取所有工作区
    pub fn get_all_workspaces(&self) -> Result<Vec<Workspace>, String> {
        let config = self.repository.read()?;
        Ok(config.workspaces)
    }

    /// 获取最近工作区
    pub fn get_last_workspace(&self) -> Result<Option<Workspace>, String> {
        let config = self.repository.read()?;
        if let Some(id) = config.last_workspace_id {
            Ok(config.workspaces.iter().find(|w| w.id == id).cloned())
        } else {
            Ok(None)
        }
    }

    /// 切换工作区
    pub fn switch_workspace(&self, id: String) -> Result<Workspace, String> {
        let mut config = self.repository.read()?;

        AppConfigDomainService::validate_switch(&config, &id)?;

        let workspace = config
            .workspaces
            .iter()
            .find(|w| w.id == id)
            .cloned()
            .ok_or_else(|| "工作区不存在".to_string())?;

        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        for w in &mut config.workspaces {
            if w.id == id {
                w.last_opened = now.clone();
                break;
            }
        }

        config.last_workspace_id = Some(id);
        self.repository.write(&config)?;

        Ok(workspace)
    }

    /// 删除工作区
    pub fn delete_workspace(&self, id: String) -> Result<(), String> {
        let mut config = self.repository.read()?;

        AppConfigDomainService::validate_delete(&config, &id)?;

        config.workspaces.retain(|w| w.id != id);

        if config.last_workspace_id == Some(id) {
            config.last_workspace_id = config.workspaces.first().map(|w| w.id.clone());
        }

        self.repository.write(&config)?;
        Ok(())
    }

    /// 更新工作区
    pub fn update_workspace(
        &self,
        id: String,
        name: String,
        description: String,
    ) -> Result<(), String> {
        let mut config = self.repository.read()?;

        for w in &mut config.workspaces {
            if w.id == id {
                w.name = name;
                w.description = description;
                break;
            }
        }

        self.repository.write(&config)?;
        Ok(())
    }

    /// 设置最后打开的接口
    pub fn set_last_api(&self, workspace_id: String, api_id: String) -> Result<(), String> {
        let mut config = self.repository.read()?;

        for w in &mut config.workspaces {
            if w.id == workspace_id {
                w.last_api_id = Some(api_id);
                break;
            }
        }

        self.repository.write(&config)?;
        Ok(())
    }

    /// 获取最后打开的接口 ID
    pub fn get_last_api_id(&self, workspace_id: &str) -> Result<Option<String>, String> {
        let config = self.repository.read()?;
        Ok(config
            .workspaces
            .iter()
            .find(|w| w.id == workspace_id)
            .and_then(|w| w.last_api_id.clone()))
    }

    /// 设置最后打开的工作区
    pub fn set_last_workspace(&self, workspace_id: String) -> Result<(), String> {
        let mut config = self.repository.read()?;

        AppConfigDomainService::validate_switch(&config, &workspace_id)?;

        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        for w in &mut config.workspaces {
            if w.id == workspace_id {
                w.last_opened = now;
                break;
            }
        }

        config.last_workspace_id = Some(workspace_id);
        self.repository.write(&config)?;
        Ok(())
    }

    /// 工作区排序
    pub fn reorder_workspaces(&self, workspace_id: String, new_index: usize) -> Result<(), String> {
        let mut config = self.repository.read()?;

        let current_index = config
            .workspaces
            .iter()
            .position(|w| w.id == workspace_id)
            .ok_or_else(|| "工作区不存在".to_string())?;

        if current_index != new_index {
            let workspace = config.workspaces.remove(current_index);
            let insert_index = new_index.min(config.workspaces.len());
            config.workspaces.insert(insert_index, workspace);
            self.repository.write(&config)?;
        }

        Ok(())
    }
}
