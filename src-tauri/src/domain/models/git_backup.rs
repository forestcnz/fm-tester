use serde::{Deserialize, Serialize};

/// Git 备份文件信息（用于列举备份）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBackupFile {
    pub workspace_name: String,
    pub file_name: String,
    pub timestamp: String,
    pub size: u64,
}

/// Git 备份配置视图（用于前端展示，密码脱敏）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBackupSettingsView {
    pub repo_url: String,
    pub branch: String,
    pub username: String,
    pub has_password: bool,
    pub auto_backup_enabled: bool,
    pub auto_backup_time: String,
    pub auto_backup_workspace_ids: Vec<String>,
}
