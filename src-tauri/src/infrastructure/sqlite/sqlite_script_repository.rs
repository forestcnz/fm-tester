//! SQLite 脚本仓储实现
//!
//! 为 local 工作区提供脚本数据的 SQLite 存储。
//! 脚本内容直接存储在 scripts 表中，不再使用独立的 .js 文件。

use crate::domain::models::{ScriptIndexEntry, ScriptKind, ScriptTargetType, ScriptsConfig};
use crate::domain::repositories::ScriptRepository;
use crate::infrastructure::data_dir;
use crate::infrastructure::sqlite::connection::with_connection;
use crate::repo_error;
use rusqlite::params;
use std::path::PathBuf;

/// SQLite 脚本仓储
///
/// 使用 SQLite 数据库存储脚本索引和脚本内容。
/// 替代 TOML + .js 文件存储方式（仅用于 local 工作区）。
pub struct SqliteScriptRepository;

impl SqliteScriptRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SqliteScriptRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteScriptRepository {
    fn target_type_to_str(tt: &ScriptTargetType) -> &'static str {
        match tt {
            ScriptTargetType::Api => "api",
            ScriptTargetType::Collection => "collection",
            ScriptTargetType::Workspace => "workspace",
            ScriptTargetType::Environment => "environment",
        }
    }

    fn str_to_target_type(s: &str) -> Option<ScriptTargetType> {
        match s {
            "api" => Some(ScriptTargetType::Api),
            "collection" => Some(ScriptTargetType::Collection),
            "workspace" => Some(ScriptTargetType::Workspace),
            "environment" => Some(ScriptTargetType::Environment),
            _ => None,
        }
    }

    fn kind_to_str(k: &ScriptKind) -> &'static str {
        match k {
            ScriptKind::Pre => "pre",
            ScriptKind::Post => "post",
        }
    }

    fn str_to_kind(s: &str) -> Option<ScriptKind> {
        match s {
            "pre" => Some(ScriptKind::Pre),
            "post" => Some(ScriptKind::Post),
            _ => None,
        }
    }

    fn row_to_raw(
        row: &rusqlite::Row,
    ) -> Result<(String, Option<String>, String, String), rusqlite::Error> {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    }

    fn raw_to_entry(
        target_type_str: String,
        target_id: Option<String>,
        script_kind_str: String,
        filename: String,
    ) -> Result<ScriptIndexEntry, String> {
        let target_type = Self::str_to_target_type(&target_type_str)
            .ok_or_else(|| repo_error!("无效的脚本目标类型: {}", target_type_str))?;
        let script_kind = Self::str_to_kind(&script_kind_str)
            .ok_or_else(|| repo_error!("无效的脚本类型: {}", script_kind_str))?;
        Ok(ScriptIndexEntry {
            target_type,
            target_id,
            script_kind,
            file: filename,
        })
    }

    fn generate_id(entry: &ScriptIndexEntry) -> String {
        let tt = Self::target_type_to_str(&entry.target_type);
        let k = Self::kind_to_str(&entry.script_kind);
        match &entry.target_id {
            Some(tid) => format!("script_{}_{}_{}", tt, tid, k),
            None => format!("script_{}_{}", tt, k),
        }
    }
}

impl ScriptRepository for SqliteScriptRepository {
    fn get_scripts_dir(&self, workspace_id: &str) -> PathBuf {
        data_dir::get_workspace_db_path(workspace_id)
    }

    fn get_config_path(&self, workspace_id: &str) -> PathBuf {
        data_dir::get_workspace_db_path(workspace_id)
    }

    fn read_config(&self, workspace_id: &str) -> Result<ScriptsConfig, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare("SELECT target_type, target_id, script_kind, filename FROM scripts")
                .map_err(|e| repo_error!("准备查询脚本失败: {}", e))?;

            let rows: Vec<(String, Option<String>, String, String)> = stmt
                .query_map([], Self::row_to_raw)
                .map_err(|e| repo_error!("查询脚本失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析脚本行数据失败: {}", e))?;

            let entries: Vec<ScriptIndexEntry> = rows
                .into_iter()
                .map(|(tt, tid, sk, f)| Self::raw_to_entry(tt, tid, sk, f))
                .collect::<Result<Vec<_>, String>>()?;

            Ok(ScriptsConfig { scripts: entries })
        })
    }

    fn write_config(&self, workspace_id: &str, config: &ScriptsConfig) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let existing_content: std::collections::HashMap<String, String> = {
                let mut map = std::collections::HashMap::new();
                let mut stmt = conn
                    .prepare("SELECT target_type, target_id, script_kind, content FROM scripts")
                    .map_err(|e| repo_error!("准备查询脚本内容失败: {}", e))?;

                let rows = stmt
                    .query_map([], |row| {
                        let tt: String = row.get(0)?;
                        let tid: Option<String> = row.get(1)?;
                        let sk: String = row.get(2)?;
                        let content: String = row.get(3)?;
                        let key = match tid {
                            Some(id) => format!("{}_{}_{}", tt, id, sk),
                            None => format!("{}_{}", tt, sk),
                        };
                        Ok((key, content))
                    })
                    .map_err(|e| repo_error!("查询脚本内容失败: {}", e))?;

                for row in rows {
                    let (key, content) =
                        row.map_err(|e| repo_error!("解析脚本内容行失败: {}", e))?;
                    map.insert(key, content);
                }
                map
            };

            // 使用 unchecked_transaction（&self 版本），失败时 Drop 自动 ROLLBACK
            let tx = conn
                .unchecked_transaction()
                .map_err(|e| repo_error!("开始事务失败: {}", e))?;

            tx.execute("DELETE FROM scripts", [])
                .map_err(|e| repo_error!("清除脚本失败: {}", e))?;

            for entry in &config.scripts {
                let id = Self::generate_id(entry);
                let target_type = Self::target_type_to_str(&entry.target_type);
                let script_kind = Self::kind_to_str(&entry.script_kind);
                let filename = &entry.file;

                let content_key = match &entry.target_id {
                    Some(tid) => format!("{}_{}_{}", target_type, tid, script_kind),
                    None => format!("{}_{}", target_type, script_kind),
                };
                let content = existing_content
                    .get(&content_key)
                    .cloned()
                    .unwrap_or_default();

                tx.execute(
                    "INSERT OR REPLACE INTO scripts (id, target_type, target_id, script_kind, filename, content) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![id, target_type, entry.target_id, script_kind, filename, content],
                )
                .map_err(|e| repo_error!("插入脚本索引失败: {}", e))?;
            }

            // 显式提交（失败时 tx Drop 自动 ROLLBACK，不会卡住后续事务）
            tx.commit()
                .map_err(|e| repo_error!("提交事务失败: {}", e))?;

            Ok(())
        })
    }

    fn read_script(&self, workspace_id: &str, filename: &str) -> Result<String, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let result = conn.query_row(
                "SELECT content FROM scripts WHERE filename = ?1",
                params![filename],
                |row| row.get::<_, Option<String>>(0),
            );

            match result {
                Ok(Some(content)) => Ok(content),
                Ok(None) => Ok(String::new()),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(String::new()),
                Err(e) => Err(repo_error!("查询脚本内容失败: {}", e)),
            }
        })
    }

    fn write_script(
        &self,
        workspace_id: &str,
        filename: &str,
        content: &str,
    ) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let result = conn.query_row(
                "SELECT id, target_type, target_id, script_kind FROM scripts WHERE filename = ?1",
                params![filename],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            );

            match result {
                Ok((id, _tt, _tid, _sk)) => {
                    conn.execute(
                        "UPDATE scripts SET content = ?1 WHERE id = ?2",
                        params![content, id],
                    )
                    .map_err(|e| repo_error!("更新脚本内容失败: {}", e))?;
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    let parts: Vec<&str> = filename.splitn(3, '_').collect();
                    if parts.len() < 2 {
                        return Err(repo_error!("无法从文件名推断脚本元数据: {}", filename));
                    }
                    let target_type_str = parts[0];
                    let rest = &filename[parts[0].len() + 1..];

                    let target_type = Self::str_to_target_type(target_type_str)
                        .ok_or_else(|| repo_error!("无效的 target_type: {}", target_type_str))?;

                    let kind_suffix = rest.strip_suffix(".js").unwrap_or(rest);

                    let (target_id, script_kind_str) =
                        if let Some(last_underscore) = kind_suffix.rfind('_') {
                            let tid = &kind_suffix[..last_underscore];
                            let kind = &kind_suffix[last_underscore + 1..];
                            (Some(tid.to_string()), kind)
                        } else {
                            (None, kind_suffix)
                        };

                    let script_kind = Self::str_to_kind(script_kind_str)
                        .ok_or_else(|| repo_error!("无效的 script_kind: {}", script_kind_str))?;

                    let entry = ScriptIndexEntry {
                        target_type,
                        target_id,
                        script_kind,
                        file: filename.to_string(),
                    };
                    let id = Self::generate_id(&entry);
                    let target_type_db = Self::target_type_to_str(&entry.target_type);
                    let script_kind_db = Self::kind_to_str(&entry.script_kind);

                    conn.execute(
                        "INSERT OR REPLACE INTO scripts \
                         (id, target_type, target_id, script_kind, filename, content) \
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![
                            id,
                            target_type_db,
                            entry.target_id,
                            script_kind_db,
                            filename,
                            content
                        ],
                    )
                    .map_err(|e| repo_error!("插入脚本失败: {}", e))?;
                }
                Err(e) => return Err(repo_error!("查询脚本元数据失败: {}", e)),
            }

            Ok(())
        })
    }

    fn delete_script(&self, workspace_id: &str, filename: &str) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute("DELETE FROM scripts WHERE filename = ?1", params![filename])
                .map_err(|e| repo_error!("删除脚本失败: {}", e))?;

            Ok(())
        })
    }

    fn get_all_entries(&self, workspace_id: &str) -> Result<Vec<ScriptIndexEntry>, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare("SELECT target_type, target_id, script_kind, filename FROM scripts")
                .map_err(|e| repo_error!("准备查询脚本条目失败: {}", e))?;

            let rows: Vec<(String, Option<String>, String, String)> = stmt
                .query_map([], Self::row_to_raw)
                .map_err(|e| repo_error!("查询脚本条目失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析脚本条目行数据失败: {}", e))?;

            let entries: Vec<ScriptIndexEntry> = rows
                .into_iter()
                .map(|(tt, tid, sk, f)| Self::raw_to_entry(tt, tid, sk, f))
                .collect::<Result<Vec<_>, String>>()?;

            Ok(entries)
        })
    }

    fn find_entry(
        &self,
        workspace_id: &str,
        target_type: &str,
        target_id: Option<&str>,
        script_kind: &str,
    ) -> Result<Option<ScriptIndexEntry>, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let result = if let Some(tid) = target_id {
                conn.query_row(
                    "SELECT target_type, target_id, script_kind, filename FROM scripts \
                     WHERE target_type = ?1 AND target_id = ?2 AND script_kind = ?3",
                    params![target_type, tid, script_kind],
                    Self::row_to_raw,
                )
            } else {
                conn.query_row(
                    "SELECT target_type, target_id, script_kind, filename FROM scripts \
                     WHERE target_type = ?1 AND target_id IS NULL AND script_kind = ?2",
                    params![target_type, script_kind],
                    Self::row_to_raw,
                )
            };

            match result {
                Ok((tt, tid, sk, f)) => Self::raw_to_entry(tt, tid, sk, f).map(Some),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(repo_error!("查询脚本条目失败: {}", e)),
            }
        })
    }

    fn delete_entries_by_target(
        &self,
        workspace_id: &str,
        target_type: &str,
        target_id: Option<&str>,
    ) -> Result<Vec<ScriptIndexEntry>, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let entries: Vec<ScriptIndexEntry> = if let Some(tid) = target_id {
                let mut stmt = conn
                    .prepare(
                        "SELECT target_type, target_id, script_kind, filename FROM scripts \
                         WHERE target_type = ?1 AND target_id = ?2",
                    )
                    .map_err(|e| repo_error!("准备查询待删除脚本条目失败: {}", e))?;

                let rows: Vec<(String, Option<String>, String, String)> = stmt
                    .query_map(params![target_type, tid], Self::row_to_raw)
                    .map_err(|e| repo_error!("查询待删除脚本条目失败: {}", e))?
                    .collect::<Result<Vec<_>, rusqlite::Error>>()
                    .map_err(|e| repo_error!("解析待删除脚本行数据失败: {}", e))?;

                rows.into_iter()
                    .map(|(tt, t_id, sk, f)| Self::raw_to_entry(tt, t_id, sk, f))
                    .collect::<Result<Vec<_>, String>>()?
            } else {
                let mut stmt = conn
                    .prepare(
                        "SELECT target_type, target_id, script_kind, filename FROM scripts \
                         WHERE target_type = ?1 AND target_id IS NULL",
                    )
                    .map_err(|e| repo_error!("准备查询待删除脚本条目失败: {}", e))?;

                let rows: Vec<(String, Option<String>, String, String)> = stmt
                    .query_map(params![target_type], Self::row_to_raw)
                    .map_err(|e| repo_error!("查询待删除脚本条目失败: {}", e))?
                    .collect::<Result<Vec<_>, rusqlite::Error>>()
                    .map_err(|e| repo_error!("解析待删除脚本行数据失败: {}", e))?;

                rows.into_iter()
                    .map(|(tt, t_id, sk, f)| Self::raw_to_entry(tt, t_id, sk, f))
                    .collect::<Result<Vec<_>, String>>()?
            };

            if let Some(tid) = target_id {
                conn.execute(
                    "DELETE FROM scripts WHERE target_type = ?1 AND target_id = ?2",
                    params![target_type, tid],
                )
                .map_err(|e| repo_error!("删除脚本条目失败: {}", e))?;
            } else {
                conn.execute(
                    "DELETE FROM scripts WHERE target_type = ?1 AND target_id IS NULL",
                    params![target_type],
                )
                .map_err(|e| repo_error!("删除脚本条目失败: {}", e))?;
            }

            Ok(entries)
        })
    }
}
