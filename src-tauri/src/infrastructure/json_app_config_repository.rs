//! JSON 应用配置仓储实现
//!
//! 全局配置存储在 {APP_DATA}/config.json，包含 settings + workspaces。

use crate::domain::models::AppConfig;
use crate::domain::repositories::AppConfigRepository;
use crate::infrastructure::data_dir;
use std::fs;

pub struct JsonAppConfigRepository;

impl JsonAppConfigRepository {
    pub fn new() -> Result<Self, String> {
        let path = data_dir::get_config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建数据目录失败: {}", e))?;
        }
        Ok(Self)
    }
}

impl Default for JsonAppConfigRepository {
    fn default() -> Self {
        Self::new().expect("无法初始化应用配置仓储")
    }
}

impl AppConfigRepository for JsonAppConfigRepository {
    fn read(&self) -> Result<AppConfig, String> {
        let path = data_dir::get_config_path();

        if !path.exists() {
            return Ok(AppConfig::default());
        }

        let content = fs::read_to_string(&path).map_err(|e| format!("读取配置文件失败: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("反序列化配置失败: {}", e))
    }

    fn write(&self, config: &AppConfig) -> Result<(), String> {
        let path = data_dir::get_config_path();
        let json_content =
            serde_json::to_string_pretty(config).map_err(|e| format!("序列化配置失败: {}", e))?;

        // 原子写入：先写临时文件，再 rename 替换。
        // 避免写到一半进程崩溃/断电导致配置文件截断损坏。
        let mut tmp_path = path.clone();
        let tmp_ext = ".tmp";
        tmp_path.set_extension(format!(
            "{}{}",
            tmp_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("json"),
            tmp_ext
        ));

        fs::write(&tmp_path, &json_content).map_err(|e| format!("写入配置临时文件失败: {}", e))?;

        // rename 在同分区是原子操作
        fs::rename(&tmp_path, &path).map_err(|e| format!("替换配置文件失败: {}", e))?;

        Ok(())
    }
}
