//! SQLite 文档仓储实现（精简版）
//!
//! 使用 docs 表存储文档索引和内容（合并 doc_index 和 doc_content）。

use crate::domain::models::{DocIndex, DocIndexEntry};
use crate::domain::repositories::MdRepository;
use crate::infrastructure::sqlite::connection::{with_connection, with_transaction};
use crate::repo_error;
use rusqlite::params;

/// SQLite 文档仓储
pub struct SqliteMdRepository;

impl SqliteMdRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SqliteMdRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MdRepository for SqliteMdRepository {
    fn read_doc_index(&self, workspace_id: &str) -> Result<DocIndex, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare("SELECT api_id, updated_at FROM docs")
                .map_err(|e| repo_error!("准备查询文档索引失败: {}", e))?;

            let entries: Vec<DocIndexEntry> = stmt
                .query_map([], |row| {
                    Ok(DocIndexEntry {
                        api_id: row.get(0)?,
                        updated_at: row.get(1)?,
                    })
                })
                .map_err(|e| repo_error!("查询文档索引失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析文档索引行数据失败: {}", e))?;

            Ok(DocIndex { entries })
        })
    }

    fn write_doc_index(&self, workspace_id: &str, index: &DocIndex) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_transaction(&ws, |conn| {
            conn.execute("DELETE FROM docs", [])
                .map_err(|e| repo_error!("清除文档失败: {}", e))?;

            for entry in &index.entries {
                conn.execute(
                    "INSERT INTO docs (api_id, updated_at, content) VALUES (?1, ?2, '')",
                    params![entry.api_id, entry.updated_at],
                )
                .map_err(|e| repo_error!("插入文档索引条目失败: {}", e))?;
            }

            Ok(())
        })
    }

    fn update_doc_index(
        &self,
        workspace_id: &str,
        api_id: &str,
        updated_at: &str,
    ) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute(
                "INSERT INTO docs (api_id, updated_at, content) VALUES (?1, ?2, '') \
                 ON CONFLICT(api_id) DO UPDATE SET updated_at = excluded.updated_at",
                params![api_id, updated_at],
            )
            .map_err(|e| repo_error!("更新文档索引失败: {}", e))?;

            Ok(())
        })
    }

    fn get_doc_index_entry(
        &self,
        workspace_id: &str,
        api_id: &str,
    ) -> Result<Option<DocIndexEntry>, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let result = conn.query_row(
                "SELECT api_id, updated_at FROM docs WHERE api_id = ?1",
                params![api_id],
                |row| {
                    Ok(DocIndexEntry {
                        api_id: row.get(0)?,
                        updated_at: row.get(1)?,
                    })
                },
            );

            match result {
                Ok(entry) => Ok(Some(entry)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(repo_error!("查询文档索引条目失败: {}", e)),
            }
        })
    }

    fn read_api_doc(&self, workspace_id: &str, api_id: &str) -> Result<String, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let result = conn.query_row(
                "SELECT content FROM docs WHERE api_id = ?1",
                params![api_id],
                |row| row.get::<_, String>(0),
            );

            match result {
                Ok(content) => Ok(content),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(String::new()),
                Err(e) => Err(repo_error!("读取API文档失败: {}", e)),
            }
        })
    }

    fn write_api_doc(&self, workspace_id: &str, api_id: &str, content: &str) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let updated_at = chrono::Local::now().to_rfc3339();
            conn.execute(
                "INSERT INTO docs (api_id, updated_at, content) VALUES (?1, ?2, ?3) \
                 ON CONFLICT(api_id) DO UPDATE SET content = excluded.content, updated_at = excluded.updated_at",
                params![api_id, updated_at, content],
            )
            .map_err(|e| repo_error!("写入API文档失败: {}", e))?;

            Ok(())
        })
    }
}
