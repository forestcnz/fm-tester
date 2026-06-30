use serde::{Deserialize, Serialize};

/// AI 设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSettings {
    #[serde(default = "default_ai_endpoint")]
    pub api_endpoint: String,
    #[serde(default)]
    pub encrypted_api_key: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub custom_headers: Vec<Header>,
    #[serde(default = "default_ai_timeout")]
    pub timeout: u64,
}

fn default_ai_endpoint() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_ai_timeout() -> u64 {
    600
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            api_endpoint: "https://api.openai.com/v1".to_string(),
            encrypted_api_key: "".to_string(),
            model: "".to_string(),
            custom_headers: Vec::new(),
            timeout: 600,
        }
    }
}

impl AiSettings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.api_endpoint.is_empty()
            && !self.api_endpoint.starts_with("http://")
            && !self.api_endpoint.starts_with("https://")
        {
            return Err("AI API 端点必须是有效的 URL".to_string());
        }
        if self.timeout < 1 {
            return Err("AI 超时时间必须大于 0".to_string());
        }
        Ok(())
    }

    pub fn is_api_key_set(&self) -> bool {
        !self.encrypted_api_key.is_empty()
    }
}

/// Git 工作区备份设置（全局备份设备配置）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBackupSettings {
    #[serde(default)]
    pub repo_url: String,
    #[serde(default = "default_git_branch")]
    pub branch: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub encrypted_password: String,
    #[serde(default)]
    pub auto_backup_enabled: bool,
    #[serde(default = "default_auto_backup_time")]
    pub auto_backup_time: String,
    /// 自动备份的目标工作区 id 列表（仅这些工作区会被自动备份；为空则不备份任何工作区）
    #[serde(default)]
    pub auto_backup_workspace_ids: Vec<String>,
}

fn default_git_branch() -> String {
    "master".to_string()
}

fn default_auto_backup_time() -> String {
    "03:00".to_string()
}

impl Default for GitBackupSettings {
    fn default() -> Self {
        Self {
            repo_url: String::new(),
            branch: "master".to_string(),
            username: String::new(),
            encrypted_password: String::new(),
            auto_backup_enabled: false,
            auto_backup_time: "03:00".to_string(),
            auto_backup_workspace_ids: Vec::new(),
        }
    }
}

impl GitBackupSettings {
    /// 是否已配置（仓库地址非空即为已配置）
    pub fn is_configured(&self) -> bool {
        !self.repo_url.trim().is_empty()
    }

    /// 是否已设置凭据
    pub fn has_credentials(&self) -> bool {
        !self.encrypted_password.is_empty()
    }
}

/// 外观设置（墨砚 Inkwell）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    #[serde(default = "default_theme_id")]
    pub theme_id: String,
    #[serde(default = "default_font_size")]
    pub font_size: u8,
    #[serde(default = "default_mono_font")]
    pub mono_font: String,
    #[serde(default = "default_density")]
    pub density: String,
    #[serde(default = "default_animations")]
    pub animations: bool,
}

fn default_theme_id() -> String {
    "paper".to_string()
}
fn default_font_size() -> u8 {
    13
}
fn default_mono_font() -> String {
    "IBM Plex Mono".to_string()
}
fn default_density() -> String {
    "comfortable".to_string()
}
fn default_animations() -> bool {
    true
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme_id: "paper".to_string(),
            font_size: 13,
            mono_font: "IBM Plex Mono".to_string(),
            density: "comfortable".to_string(),
            animations: true,
        }
    }
}

/// 行为设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSettings {
    #[serde(default = "default_restore_workspace")]
    pub restore_workspace_on_start: bool,
    #[serde(default = "default_keep_tab")]
    pub keep_tab_on_send: bool,
    #[serde(default)]
    pub auto_save_on_send: bool,
}

fn default_restore_workspace() -> bool {
    true
}
fn default_keep_tab() -> bool {
    true
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            restore_workspace_on_start: true,
            keep_tab_on_send: true,
            auto_save_on_send: false,
        }
    }
}

/// 全局应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_timeout")]
    pub request_timeout: u64,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub ai: AiSettings,
    #[serde(default)]
    pub git_backup: GitBackupSettings,
    #[serde(default)]
    pub appearance: AppearanceSettings,
    #[serde(default)]
    pub behavior: BehaviorSettings,
}

fn default_timeout() -> u64 {
    60
}

fn default_language() -> String {
    "zh-CN".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            request_timeout: 60,
            language: "zh-CN".to_string(),
            ai: AiSettings::default(),
            git_backup: GitBackupSettings::default(),
            appearance: AppearanceSettings::default(),
            behavior: BehaviorSettings::default(),
        }
    }
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.request_timeout < 1 {
            return Err("请求超时时间必须大于 0".to_string());
        }
        if !["zh-CN", "en"].contains(&self.language.as_str()) {
            return Err("语言设置必须是 zh-CN 或 en".to_string());
        }
        // 外观校验
        if !["paper", "dark", "one-dark"].contains(&self.appearance.theme_id.as_str()) {
            return Err("主题必须是 paper 或 dark".to_string());
        }
        if !(11..=16).contains(&self.appearance.font_size) {
            return Err("字号必须在 11-16 之间".to_string());
        }
        if !["comfortable", "compact"].contains(&self.appearance.density.as_str()) {
            return Err("界面密度必须是 comfortable 或 compact".to_string());
        }
        self.ai.validate()?;
        Ok(())
    }
}

// 引用 Header，需要从 collection.rs 导入
use super::Header;
