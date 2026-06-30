//! SQLite 保存响应仓储实现（简化版）
//!
//! 只保存基本信息和 MD 文档内容。

use crate::domain::models::{SavedResponse, SavedResponseIndexEntry, SavedResponsesIndex};
use crate::domain::repositories::ResponseRepository;
use crate::infrastructure::sqlite::connection::with_connection;
use crate::repo_error;
use rusqlite::params;

/// SQLite 保存响应仓储
pub struct SqliteResponseRepository;

impl SqliteResponseRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SqliteResponseRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseRepository for SqliteResponseRepository {
    fn get_index(&self, workspace_id: &str) -> Result<SavedResponsesIndex, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, created_at, api_id FROM saved_responses ORDER BY created_at DESC",
                )
                .map_err(|e| repo_error!("准备查询响应索引失败: {}", e))?;

            let responses: Vec<SavedResponseIndexEntry> = stmt
                .query_map([], |row| {
                    Ok(SavedResponseIndexEntry {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        created_at: row.get(2)?,
                        api_id: row.get(3)?,
                    })
                })
                .map_err(|e| repo_error!("查询响应索引失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析响应索引行数据失败: {}", e))?;

            Ok(SavedResponsesIndex { responses })
        })
    }

    fn save_index(&self, workspace_id: &str, index: &SavedResponsesIndex) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            for entry in &index.responses {
                let exists: bool = conn
                    .query_row(
                        "SELECT COUNT(*) > 0 FROM saved_responses WHERE id = ?1",
                        params![entry.id],
                        |row| row.get::<_, bool>(0),
                    )
                    .map_err(|e| repo_error!("验证索引条目存在失败: {}", e))?;

                if !exists {
                    return Err(repo_error!("索引条目{} 对应的响应记录不存在", entry.id));
                }
            }

            Ok(())
        })
    }

    fn get(&self, workspace_id: &str, id: &str) -> Result<Option<SavedResponse>, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let result = conn.query_row(
                "SELECT id, name, created_at, api_id, doc_content FROM saved_responses WHERE id = ?1",
                params![id],
                |row| {
                    Ok(SavedResponse {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        created_at: row.get(2)?,
                        api_id: row.get(3)?,
                        doc_content: row.get(4)?,
                    })
                },
            );

            match result {
                Ok(response) => Ok(Some(response)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(repo_error!("查询保存响应失败: {}", e)),
            }
        })
    }

    fn save(
        &self,
        workspace_id: &str,
        response: &SavedResponse,
        _index_entry: &SavedResponseIndexEntry,
    ) -> Result<(), String> {
        if response.id.is_empty() {
            return Err(repo_error!("响应 ID 不能为空"));
        }
        if response.name.is_empty() {
            return Err(repo_error!("响应名称不能为空"));
        }

        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute(
                "INSERT OR REPLACE INTO saved_responses (id, name, created_at, api_id, doc_content)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    response.id,
                    response.name,
                    response.created_at,
                    response.api_id,
                    response.doc_content,
                ],
            )
            .map_err(|e| repo_error!("保存响应失败: {}", e))?;

            Ok(())
        })
    }

    fn delete(&self, workspace_id: &str, id: &str) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute("DELETE FROM saved_responses WHERE id = ?1", params![id])
                .map_err(|e| repo_error!("删除保存响应失败: {}", e))?;
            Ok(())
        })
    }

    fn filter_by_api(
        &self,
        workspace_id: &str,
        api_id: &str,
    ) -> Result<Vec<SavedResponseIndexEntry>, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, created_at, api_id FROM saved_responses WHERE api_id = ?1 ORDER BY created_at DESC",
                )
                .map_err(|e| repo_error!("准备查询响应索引失败: {}", e))?;

            let responses: Vec<SavedResponseIndexEntry> = stmt
                .query_map(params![api_id], |row| {
                    Ok(SavedResponseIndexEntry {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        created_at: row.get(2)?,
                        api_id: row.get(3)?,
                    })
                })
                .map_err(|e| repo_error!("查询响应索引失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析响应索引行数据失败: {}", e))?;

            Ok(responses)
        })
    }
}
