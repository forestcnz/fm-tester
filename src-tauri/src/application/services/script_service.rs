//! 脚本应用服务
//!
//! 协调仓储和领域服务进行脚本管理操作，通过仓储工厂动态获取仓储。

use crate::domain::models::{ScriptIndexEntry, ScriptKind, ScriptTargetType, ScriptsConfig};
use crate::domain::services::ScriptDomainService;
use crate::infrastructure::RepositoryFactory;

/// 脚本应用服务
pub struct ScriptApplicationService;

impl ScriptApplicationService {
    /// 保存脚本（内容 + 索引）
    pub fn save(
        workspace_id: &str,
        target_type: ScriptTargetType,
        target_id: Option<String>,
        script_kind: ScriptKind,
        content: &str,
    ) -> Result<(), String> {
        let repo = RepositoryFactory::get_script_repository();

        // 生成文件名（领域服务）
        let filename = ScriptDomainService::generate_script_filename(
            target_type.clone(),
            target_id.as_deref(),
            script_kind.clone(),
        );

        // 保存脚本文件
        repo.write_script(workspace_id, &filename, content)?;

        // 更新索引
        let mut config = repo.read_config(workspace_id)?;
        // 查找是否已存在
        let existing_index = config.scripts.iter().position(|s| {
            s.target_type == target_type && s.target_id == target_id && s.script_kind == script_kind
        });

        let entry = ScriptIndexEntry {
            target_type,
            target_id,
            script_kind,
            file: filename,
        };

        if let Some(index) = existing_index {
            config.scripts[index] = entry;
        } else {
            config.scripts.push(entry);
        }

        // 保存索引
        repo.write_config(workspace_id, &config)?;

        Ok(())
    }

    /// 获取脚本内容
    pub fn get(
        workspace_id: &str,
        target_type: ScriptTargetType,
        target_id: Option<String>,
        script_kind: ScriptKind,
    ) -> Result<String, String> {
        let repo = RepositoryFactory::get_script_repository();

        let config = repo.read_config(workspace_id)?;
        let entry = config.scripts.iter().find(|s| {
            s.target_type == target_type && s.target_id == target_id && s.script_kind == script_kind
        });

        if let Some(entry) = entry {
            repo.read_script(workspace_id, &entry.file) // 直接用 entry.file，不再去掉前缀
        } else {
            Ok(String::new())
        }
    }

    /// 删除脚本（文件 + 索引）
    pub fn delete(
        workspace_id: &str,
        target_type: ScriptTargetType,
        target_id: Option<String>,
        script_kind: ScriptKind,
    ) -> Result<(), String> {
        let repo = RepositoryFactory::get_script_repository();

        let mut config = repo.read_config(workspace_id)?;

        let index = config.scripts.iter().position(|s| {
            s.target_type == target_type && s.target_id == target_id && s.script_kind == script_kind
        });

        if let Some(index) = index {
            let entry = &config.scripts[index];

            // 删除文件
            repo.delete_script(workspace_id, &entry.file)?; // 直接用 entry.file

            // 从索引移除
            config.scripts.remove(index);

            // 保存索引
            repo.write_config(workspace_id, &config)?;
        }

        Ok(())
    }

    /// 删除目标的所有脚本（删除 api/collection 时调用）
    pub fn delete_by_target(
        workspace_id: &str,
        target_type: ScriptTargetType,
        target_id: Option<String>,
    ) -> Result<(), String> {
        let repo = RepositoryFactory::get_script_repository();

        let target_type_str = match target_type {
            ScriptTargetType::Api => "api",
            ScriptTargetType::Collection => "collection",
            ScriptTargetType::Workspace => "workspace",
            ScriptTargetType::Environment => "environment",
        };

        repo.delete_entries_by_target(workspace_id, target_type_str, target_id.as_deref())?;

        Ok(())
    }

    /// 获取所有脚本列表
    pub fn get_all(workspace_id: &str) -> Result<Vec<ScriptIndexEntry>, String> {
        let repo = RepositoryFactory::get_script_repository();
        repo.get_all_entries(workspace_id)
    }

    /// 获取脚本索引配置（用于迁移等场景）
    pub fn get_config(workspace_id: &str) -> Result<ScriptsConfig, String> {
        let repo = RepositoryFactory::get_script_repository();
        repo.read_config(workspace_id)
    }

    /// 保存脚本索引配置（用于迁移等场景）
    pub fn save_config(workspace_id: &str, config: &ScriptsConfig) -> Result<(), String> {
        let repo = RepositoryFactory::get_script_repository();
        repo.write_config(workspace_id, config)
    }

    /// 读取脚本文件内容（用于迁移）
    pub fn read_file(workspace_id: &str, filename: &str) -> Result<String, String> {
        let repo = RepositoryFactory::get_script_repository();
        repo.read_script(workspace_id, filename)
    }

    /// 保存脚本文件内容（用于迁移）
    pub fn write_file(workspace_id: &str, filename: &str, content: &str) -> Result<(), String> {
        let repo = RepositoryFactory::get_script_repository();
        repo.write_script(workspace_id, filename, content)
    }

    /// 生成脚本文件名（领域服务）
    pub fn generate_filename(
        target_type: ScriptTargetType,
        target_id: Option<&str>,
        script_kind: ScriptKind,
    ) -> String {
        ScriptDomainService::generate_script_filename(target_type, target_id, script_kind)
    }
}
