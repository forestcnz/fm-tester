//! SQLite WebSocket 配置仓储实现
//!
//! 持久化 `ws_configs` 表对应的数据，遵循 DDD 分层。

use crate::domain::models::{WsConfigEntry, WsHeader, WsParam};
use crate::domain::repositories::WsConfigRepository;
use crate::infrastructure::sqlite::connection::{with_connection, with_transaction};
use crate::repo_error;
use rusqlite::params;

/// SQLite WebSocket 配置仓储
pub struct SqliteWsConfigRepository;

impl SqliteWsConfigRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SqliteWsConfigRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl WsConfigRepository for SqliteWsConfigRepository {
    fn list(&self, workspace_id: &str) -> Result<Vec<WsConfigEntry>, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, url, headers_json, params_json, created_at, updated_at \
                     FROM ws_configs ORDER BY order_index",
                )
                .map_err(|e| repo_error!("准备查询 WebSocket 配置失败: {}", e))?;

            let entries: Vec<WsConfigEntry> = stmt
                .query_map([], |row| {
                    let id: String = row.get(0)?;
                    let name: String = row.get(1)?;
                    let url: String = row.get(2)?;
                    let headers_json: String = row.get(3)?;
                    let params_json: String = row.get(4)?;
                    let created_at: String = row.get(5)?;
                    let updated_at: String = row.get(6)?;

                    // 解析 JSON 列，失败时退化为空数组（避免单行损坏影响整体读取）
                    let headers: Vec<WsHeader> =
                        serde_json::from_str(&headers_json).unwrap_or_default();
                    let params: Vec<WsParam> =
                        serde_json::from_str(&params_json).unwrap_or_default();

                    Ok(WsConfigEntry {
                        id,
                        name,
                        url,
                        headers,
                        params,
                        created_at,
                        updated_at,
                    })
                })
                .map_err(|e| repo_error!("读取 WebSocket 配置失败: {}", e))?
                .filter_map(|r| r.ok())
                .collect();

            Ok(entries)
        })
    }

    fn upsert(&self, workspace_id: &str, entry: &WsConfigEntry) -> Result<String, String> {
        let ws = workspace_id.to_string();
        let id = entry.id.clone();
        let name = entry.name.clone();
        let url = entry.url.clone();
        let headers_json = serde_json::to_string(&entry.headers)
            .map_err(|e| repo_error!("序列化 headers 失败: {}", e))?;
        let params_json = serde_json::to_string(&entry.params)
            .map_err(|e| repo_error!("序列化 params 失败: {}", e))?;
        let created_at = entry.created_at.clone();
        let updated_at = entry.updated_at.clone();

        with_transaction(&ws, |conn| {
            // 先尝试 UPDATE
            let updated = conn
                .execute(
                    "UPDATE ws_configs \
                     SET name = ?1, url = ?2, headers_json = ?3, params_json = ?4, updated_at = ?5 \
                     WHERE id = ?6",
                    params![&name, &url, &headers_json, &params_json, &updated_at, &id],
                )
                .map_err(|e| repo_error!("更新 WebSocket 配置失败: {}", e))?;

            if updated == 0 {
                // 不存在，则插入（order_index 默认 0，前端如需排序可后续扩展）
                conn.execute(
                    "INSERT INTO ws_configs \
                     (id, name, url, headers_json, params_json, created_at, updated_at, order_index) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)",
                    params![&id, &name, &url, &headers_json, &params_json, &created_at, &updated_at],
                )
                .map_err(|e| repo_error!("创建 WebSocket 配置失败: {}", e))?;
            }
            Ok(id)
        })
    }

    fn delete(&self, workspace_id: &str, id: &str) -> Result<(), String> {
        let ws = workspace_id.to_string();
        let id = id.to_string();
        with_connection(&ws, |conn| {
            conn.execute("DELETE FROM ws_configs WHERE id = ?1", params![&id])
                .map_err(|e| repo_error!("删除 WebSocket 配置失败: {}", e))?;
            Ok(())
        })
    }
}
