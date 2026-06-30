use crate::domain::models::{ScriptIndexEntry, ScriptsConfig};
use std::path::PathBuf;

/// 脚本仓储接口
///
/// 定义脚本数据的持久化操作，遵循 DDD 仓储模式。
/// 实现类负责具体的文件系统操作。
pub trait ScriptRepository {
    /// 获取脚本目录路径
    fn get_scripts_dir(&self, workspace_id: &str) -> PathBuf;

    /// 获取脚本索引文件路径
    fn get_config_path(&self, workspace_id: &str) -> PathBuf;

    /// 读取脚本索引配置
    fn read_config(&self, workspace_id: &str) -> Result<ScriptsConfig, String>;

    /// 保存脚本索引配置
    fn write_config(&self, workspace_id: &str, config: &ScriptsConfig) -> Result<(), String>;

    /// 读取脚本文件内容
    fn read_script(&self, workspace_id: &str, filename: &str) -> Result<String, String>;

    /// 保存脚本文件内容
    fn write_script(&self, workspace_id: &str, filename: &str, content: &str)
        -> Result<(), String>;

    /// 删除脚本文件
    fn delete_script(&self, workspace_id: &str, filename: &str) -> Result<(), String>;

    /// 获取所有脚本索引条目
    fn get_all_entries(&self, workspace_id: &str) -> Result<Vec<ScriptIndexEntry>, String>;

    /// 查找匹配的脚本索引条目
    fn find_entry(
        &self,
        workspace_id: &str,
        target_type: &str,
        target_id: Option<&str>,
        script_kind: &str,
    ) -> Result<Option<ScriptIndexEntry>, String>;

    /// 删除匹配的所有脚本索引条目
    fn delete_entries_by_target(
        &self,
        workspace_id: &str,
        target_type: &str,
        target_id: Option<&str>,
    ) -> Result<Vec<ScriptIndexEntry>, String>;
}
