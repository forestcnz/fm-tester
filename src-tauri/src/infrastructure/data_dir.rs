//! 数据目录管理
//!
//! 所有数据存储在应用 exe 同级 `./data/` 目录下：
//! - data/<workspace_id>/data.db：各工作区数据库
//! - data/config.json：全局配置
//!
//! 注意：加密密钥存放于 OS 标准配置目录（与数据库分离），
//! 避免数据库被复制时密钥一并泄漏，详见 `get_key_path`。

use std::path::PathBuf;

pub fn get_data_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    exe_dir.join("data")
}

pub fn get_workspace_dir(workspace_id: &str) -> PathBuf {
    get_data_dir().join(workspace_id)
}

pub fn get_workspace_db_path(workspace_id: &str) -> PathBuf {
    get_workspace_dir(workspace_id).join("data.db")
}

pub fn get_config_path() -> PathBuf {
    get_data_dir().join("config.json")
}

/// 配置目录（与 data 目录同级：./config/）
pub fn get_config_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    exe_dir.join("config")
}

/// 环境配置文件路径（./config/config.env）
pub fn get_config_env_path() -> PathBuf {
    get_config_dir().join("config.env")
}

/// 日志目录（./data/logs/）
pub fn get_logs_dir() -> PathBuf {
    get_data_dir().join("logs")
}

/// 加密密钥路径
///
/// 注意：早期实现尝试将密钥与数据库分离到 OS 配置目录（如 %APPDATA%），
/// 但部分 Windows 环境下用户对 %APPDATA% 的写入受限，会导致加密服务初始化失败、
/// 应用启动卡死。为兼容性起见，密钥仍与数据库同目录。
/// 通过 SQLite 自身的文件权限和 OS 文件系统权限保护密钥。
/// 后续可考虑结合 OS 密钥链（DPAPI / Keychain）做更严格的隔离。
pub fn get_key_path() -> PathBuf {
    get_data_dir().join(".encryption_key")
}
