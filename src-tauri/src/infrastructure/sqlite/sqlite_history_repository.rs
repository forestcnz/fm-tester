//! SQLite 历史记录仓储实现（精简版）
//!
//! 使用 JSON 列存储子表数据，减少表数量。
//! 数据按日期分组存储在 history_entries 表中。

use crate::domain::models::{FormField, Header, HistoryEntry};
use crate::domain::repositories::HistoryRepository;
use crate::infrastructure::sqlite::connection::with_connection;
use crate::repo_error;
use rusqlite::params;
use std::collections::HashMap;

/// SQLite 历史记录仓储
pub struct SqliteHistoryRepository;

impl SqliteHistoryRepository {
    pub fn new() -> Self {
        Self
    }

    fn extract_date_from_timestamp(timestamp: &str) -> String {
        use chrono::{DateTime, Local};

        if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
            dt.with_timezone(&Local).format("%Y-%m-%d").to_string()
        } else {
            Local::now().format("%Y-%m-%d").to_string()
        }
    }

    fn reconstruct_entry_from_json(
        id: String,
        method: String,
        url: String,
        resolved_url: String,
        status: i32,
        status_text: String,
        response_body: String,
        time: i32,
        size: i32,
        created_at: String,
        body: Option<String>,
        body_type: Option<String>,
        api_id: Option<String>,
        api_name: Option<String>,
        request_headers_json: String,
        response_headers_json: String,
        form_fields_json: String,
    ) -> Result<HistoryEntry, String> {
        let headers: Vec<Header> = serde_json::from_str(&request_headers_json)
            .map_err(|e| repo_error!("反序列化历史请求头失败: {}", e))?;

        let response_headers: HashMap<String, String> =
            serde_json::from_str(&response_headers_json)
                .map_err(|e| repo_error!("反序列化历史响应头失败: {}", e))?;

        let form_fields: Vec<FormField> = serde_json::from_str(&form_fields_json)
            .map_err(|e| repo_error!("反序列化表单字段失败: {}", e))?;

        Ok(HistoryEntry {
            id,
            method,
            url,
            resolved_url,
            headers,
            body,
            body_type,
            form_fields: if form_fields.is_empty() {
                None
            } else {
                Some(form_fields)
            },
            status: status as u16,
            status_text,
            response_headers,
            response_body,
            time: time as u64,
            size: size as u64,
            created_at,
            api_id,
            api_name,
        })
    }

    fn serialize_entry_to_json(entry: &HistoryEntry) -> Result<(String, String, String), String> {
        let request_headers_json = serde_json::to_string(&entry.headers)
            .map_err(|e| repo_error!("序列化请求头失败: {}", e))?;

        let response_headers_json = serde_json::to_string(&entry.response_headers)
            .map_err(|e| repo_error!("序列化响应头失败: {}", e))?;

        let form_fields_json =
            serde_json::to_string(entry.form_fields.as_ref().unwrap_or(&Vec::new()))
                .map_err(|e| repo_error!("序列化表单字段失败: {}", e))?;

        Ok((
            request_headers_json,
            response_headers_json,
            form_fields_json,
        ))
    }
}

impl Default for SqliteHistoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryRepository for SqliteHistoryRepository {
    fn list_dates(&self, workspace_id: &str) -> Result<Vec<String>, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare("SELECT DISTINCT date FROM history_entries ORDER BY date DESC")
                .map_err(|e| repo_error!("准备查询历史日期失败: {}", e))?;

            let dates: Vec<String> = stmt
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|e| repo_error!("查询历史日期失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析历史日期行数据失败: {}", e))?;

            Ok(dates)
        })
    }

    fn get_by_date(&self, workspace_id: &str, date: &str) -> Result<Vec<HistoryEntry>, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, method, url, resolved_url, status, status_text, \
                     response_body, time, size, created_at, body, body_type, api_id, api_name, \
                     request_headers_json, response_headers_json, form_fields_json \
                     FROM history_entries \
                     WHERE date = ?1 \
                     ORDER BY created_at DESC",
                )
                .map_err(|e| repo_error!("准备查询历史记录失败: {}", e))?;

            let rows: Vec<(
                String,
                String,
                String,
                String,
                i32,
                String,
                String,
                i32,
                i32,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                String,
                String,
                String,
            )> = stmt
                .query_map(params![date], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i32>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, i32>(7)?,
                        row.get::<_, i32>(8)?,
                        row.get::<_, String>(9)?,
                        row.get::<_, Option<String>>(10)?,
                        row.get::<_, Option<String>>(11)?,
                        row.get::<_, Option<String>>(12)?,
                        row.get::<_, Option<String>>(13)?,
                        row.get::<_, String>(14)?,
                        row.get::<_, String>(15)?,
                        row.get::<_, String>(16)?,
                    ))
                })
                .map_err(|e| repo_error!("查询历史记录失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析历史记录行数据失败: {}", e))?;

            let entries: Vec<HistoryEntry> = rows
                .into_iter()
                .map(
                    |(
                        id,
                        method,
                        url,
                        resolved_url,
                        status,
                        status_text,
                        response_body,
                        time,
                        size,
                        created_at,
                        body,
                        body_type,
                        api_id,
                        api_name,
                        request_headers_json,
                        response_headers_json,
                        form_fields_json,
                    )| {
                        Self::reconstruct_entry_from_json(
                            id,
                            method,
                            url,
                            resolved_url,
                            status,
                            status_text,
                            response_body,
                            time,
                            size,
                            created_at,
                            body,
                            body_type,
                            api_id,
                            api_name,
                            request_headers_json,
                            response_headers_json,
                            form_fields_json,
                        )
                    },
                )
                .collect::<Result<Vec<_>, String>>()?;

            Ok(entries)
        })
    }

    /// 获取指定接口的最近历史记录（按 created_at 倒序，限制条数）
    fn get_by_api(
        &self,
        workspace_id: &str,
        api_id: &str,
        limit: usize,
    ) -> Result<Vec<HistoryEntry>, String> {
        let ws = workspace_id.to_string();
        let limit_i = limit as i32;
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, method, url, resolved_url, status, status_text, \
                     response_body, time, size, created_at, body, body_type, api_id, api_name, \
                     request_headers_json, response_headers_json, form_fields_json \
                     FROM history_entries \
                     WHERE api_id = ?1 \
                     ORDER BY created_at DESC \
                     LIMIT ?2",
                )
                .map_err(|e| repo_error!("准备查询历史记录失败: {}", e))?;

            let rows: Vec<(
                String,
                String,
                String,
                String,
                i32,
                String,
                String,
                i32,
                i32,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                String,
                String,
                String,
            )> = stmt
                .query_map(params![api_id, limit_i], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i32>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, i32>(7)?,
                        row.get::<_, i32>(8)?,
                        row.get::<_, String>(9)?,
                        row.get::<_, Option<String>>(10)?,
                        row.get::<_, Option<String>>(11)?,
                        row.get::<_, Option<String>>(12)?,
                        row.get::<_, Option<String>>(13)?,
                        row.get::<_, String>(14)?,
                        row.get::<_, String>(15)?,
                        row.get::<_, String>(16)?,
                    ))
                })
                .map_err(|e| repo_error!("查询历史记录失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析历史记录行数据失败: {}", e))?;

            let entries: Vec<HistoryEntry> = rows
                .into_iter()
                .map(
                    |(
                        id,
                        method,
                        url,
                        resolved_url,
                        status,
                        status_text,
                        response_body,
                        time,
                        size,
                        created_at,
                        body,
                        body_type,
                        api_id,
                        api_name,
                        request_headers_json,
                        response_headers_json,
                        form_fields_json,
                    )| {
                        Self::reconstruct_entry_from_json(
                            id,
                            method,
                            url,
                            resolved_url,
                            status,
                            status_text,
                            response_body,
                            time,
                            size,
                            created_at,
                            body,
                            body_type,
                            api_id,
                            api_name,
                            request_headers_json,
                            response_headers_json,
                            form_fields_json,
                        )
                    },
                )
                .collect::<Result<Vec<_>, String>>()?;

            Ok(entries)
        })
    }

    fn get_entry(
        &self,
        workspace_id: &str,
        date: &str,
        id: &str,
    ) -> Result<Option<HistoryEntry>, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let result = conn.query_row(
                "SELECT id, method, url, resolved_url, status, status_text, \
                 response_body, time, size, created_at, body, body_type, api_id, api_name, \
                 request_headers_json, response_headers_json, form_fields_json \
                 FROM history_entries \
                 WHERE date = ?1 AND id = ?2",
                params![date, id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i32>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, i32>(7)?,
                        row.get::<_, i32>(8)?,
                        row.get::<_, String>(9)?,
                        row.get::<_, Option<String>>(10)?,
                        row.get::<_, Option<String>>(11)?,
                        row.get::<_, Option<String>>(12)?,
                        row.get::<_, Option<String>>(13)?,
                        row.get::<_, String>(14)?,
                        row.get::<_, String>(15)?,
                        row.get::<_, String>(16)?,
                    ))
                },
            );

            match result {
                Ok((
                    entry_id,
                    method,
                    url,
                    resolved_url,
                    status,
                    status_text,
                    response_body,
                    time,
                    size,
                    created_at,
                    body,
                    body_type,
                    api_id,
                    api_name,
                    request_headers_json,
                    response_headers_json,
                    form_fields_json,
                )) => Self::reconstruct_entry_from_json(
                    entry_id,
                    method,
                    url,
                    resolved_url,
                    status,
                    status_text,
                    response_body,
                    time,
                    size,
                    created_at,
                    body,
                    body_type,
                    api_id,
                    api_name,
                    request_headers_json,
                    response_headers_json,
                    form_fields_json,
                )
                .map(Some),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(repo_error!("查询历史记录详情失败: {}", e)),
            }
        })
    }

    fn save_entry(&self, workspace_id: &str, entry: &HistoryEntry) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let date = Self::extract_date_from_timestamp(&entry.created_at);

            let (request_headers_json, response_headers_json, form_fields_json) =
                Self::serialize_entry_to_json(entry)?;

            // 使用 unchecked_transaction，失败时 Drop 自动 ROLLBACK
            let tx = conn
                .unchecked_transaction()
                .map_err(|e| repo_error!("开始事务失败: {}", e))?;

            tx.execute(
                "INSERT OR REPLACE INTO history_entries \
                 (id, method, url, resolved_url, status, status_text, \
                  response_body, time, size, created_at, body, body_type, \
                  api_id, api_name, date, \
                  request_headers_json, response_headers_json, form_fields_json) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                params![
                    entry.id,
                    entry.method,
                    entry.url,
                    entry.resolved_url,
                    entry.status as i32,
                    entry.status_text,
                    entry.response_body,
                    entry.time as i32,
                    entry.size as i32,
                    entry.created_at,
                    entry.body,
                    entry.body_type,
                    entry.api_id,
                    entry.api_name,
                    date,
                    request_headers_json,
                    response_headers_json,
                    form_fields_json,
                ],
            )
            .map_err(|e| repo_error!("插入历史记录失败: {}", e))?;

            tx.commit()
                .map_err(|e| repo_error!("提交事务失败: {}", e))?;

            Ok(())
        })
    }

    fn delete_entry(&self, workspace_id: &str, _date: &str, id: &str) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute("DELETE FROM history_entries WHERE id = ?1", params![id])
                .map_err(|e| repo_error!("删除历史记录失败: {}", e))?;

            Ok(())
        })
    }

    fn clear_by_date(&self, workspace_id: &str, date: &str) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute("DELETE FROM history_entries WHERE date = ?1", params![date])
                .map_err(|e| repo_error!("清空指定日期历史记录失败: {}", e))?;

            Ok(())
        })
    }

    fn clear_all(&self, workspace_id: &str) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute("DELETE FROM history_entries", [])
                .map_err(|e| repo_error!("清空所有历史记录失败: {}", e))?;

            Ok(())
        })
    }
}
