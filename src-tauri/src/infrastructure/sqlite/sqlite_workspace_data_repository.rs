//! SQLite 工作区数据仓储实现
//!
//! 精简版：环境和 UI 状态数据存储为 JSON 列。

use crate::domain::models::{
    Cookie, CookiesConfig, Environment, EnvironmentsConfig, Header, MemoryConfig, Variable,
};
use crate::domain::repositories::WorkspaceDataRepository;
use crate::infrastructure::sqlite::connection::{with_connection, with_transaction};
use crate::repo_error;
use rusqlite::params;
use std::collections::HashMap;

/// SQLite 工作区数据仓储
pub struct SqliteWorkspaceDataRepository;

impl SqliteWorkspaceDataRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SqliteWorkspaceDataRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceDataRepository for SqliteWorkspaceDataRepository {
    fn read_environments(&self, workspace_id: &str) -> Result<EnvironmentsConfig, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, variables_json, common_headers_json FROM environments ORDER BY order_index",
                )
                .map_err(|e| repo_error!("准备查询环境失败: {}", e))?;

            let rows: Vec<(String, String, String, String)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                })
                .map_err(|e| repo_error!("查询环境失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析环境行数据失败: {}", e))?;

            let mut environments = Vec::new();
            for (id, name, variables_json, common_headers_json) in rows {
                let variables: Vec<Variable> = serde_json::from_str(&variables_json)
                    .map_err(|e| repo_error!("反序列化变量失败: {}", e))?;
                let common_headers: Option<Vec<Header>> =
                    serde_json::from_str(&common_headers_json)
                        .map_err(|e| repo_error!("反序列化公共请求头失败: {}", e))
                        .map(|h: Vec<Header>| if h.is_empty() { None } else { Some(h) })?;

                environments.push(Environment {
                    id,
                    name,
                    variables,
                    common_headers,
                });
            }

            let active_id: Option<String> = match conn.query_row(
                "SELECT active_environment_id FROM app_state WHERE id = 1",
                [],
                |row| row.get::<_, Option<String>>(0),
            ) {
                Ok(id) => id,
                Err(rusqlite::Error::QueryReturnedNoRows) => None,
                Err(e) => return Err(repo_error!("读取激活环境ID失败: {}", e)),
            };

            Ok(EnvironmentsConfig {
                environments,
                active_environment_id: active_id,
            })
        })
    }

    fn write_environments(
        &self,
        workspace_id: &str,
        config: &EnvironmentsConfig,
    ) -> Result<(), String> {
        for env in &config.environments {
            if env.id.is_empty() {
                return Err(repo_error!("环境 ID 不能为空"));
            }
            if env.name.is_empty() {
                return Err(repo_error!("环境名称不能为空"));
            }
        }

        let ws = workspace_id.to_string();
        with_transaction(&ws, |conn| {
            conn.execute("DELETE FROM environments", [])
                .map_err(|e| repo_error!("清除环境失败: {}", e))?;

            for (i, env) in config.environments.iter().enumerate() {
                let variables_json = serde_json::to_string(&env.variables)
                    .map_err(|e| repo_error!("序列化变量失败: {}", e))?;
                let common_headers_json =
                    serde_json::to_string(&env.common_headers.as_ref().unwrap_or(&Vec::new()))
                        .map_err(|e| repo_error!("序列化公共请求头失败: {}", e))?;

                conn.execute(
                    "INSERT INTO environments (id, name, variables_json, common_headers_json, order_index) \
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![env.id, env.name, variables_json, common_headers_json, i],
                )
                .map_err(|e| repo_error!("插入环境失败: {}", e))?;
            }

            conn.execute(
                "UPDATE app_state SET active_environment_id = ?1 WHERE id = 1",
                params![config.active_environment_id],
            )
            .map_err(|e| repo_error!("设置激活环境失败: {}", e))?;

            Ok(())
        })
    }

    fn find_environment_by_name(
        &self,
        workspace_id: &str,
        name: &str,
    ) -> Result<Option<Environment>, String> {
        let config = self.read_environments(workspace_id)?;
        Ok(config.environments.iter().find(|e| e.name == name).cloned())
    }

    fn read_memory(&self, workspace_id: &str) -> Result<MemoryConfig, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let result = conn.query_row(
                "SELECT expanded_ids_json, open_tabs_json, active_tab_index, request_tabs_json \
                 FROM app_state WHERE id = 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i32>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            );

            match result {
                Ok((expanded_ids_json, open_tabs_json, active_tab_index, request_tabs_json)) => {
                    let expanded_ids: Vec<String> = serde_json::from_str(&expanded_ids_json)
                        .map_err(|e| repo_error!("反序列化展开ID失败: {}", e))?;

                    let open_tabs_data: Vec<serde_json::Value> =
                        serde_json::from_str(&open_tabs_json)
                            .map_err(|e| repo_error!("反序列化标签页失败: {}", e))?;
                    let mut open_tabs = Vec::new();
                    let mut open_tab_types = Vec::new();
                    for item in open_tabs_data {
                        if let (Some(id), Some(ttype)) = (
                            item.get("id").and_then(|v| v.as_str()),
                            item.get("type").and_then(|v| v.as_str()),
                        ) {
                            open_tabs.push(id.to_string());
                            open_tab_types.push(ttype.to_string());
                        }
                    }

                    let request_tabs: HashMap<String, String> =
                        serde_json::from_str(&request_tabs_json)
                            .map_err(|e| repo_error!("反序列化请求标签页失败: {}", e))?;

                    Ok(MemoryConfig {
                        expanded_ids,
                        open_tabs,
                        open_tab_types,
                        active_tab_index: active_tab_index as usize,
                        request_tabs,
                    })
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(MemoryConfig::default()),
                Err(e) => Err(repo_error!("读取记忆配置失败: {}", e)),
            }
        })
    }

    fn write_memory(&self, workspace_id: &str, config: &MemoryConfig) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let expanded_ids_json = serde_json::to_string(&config.expanded_ids)
                .map_err(|e| repo_error!("序列化展开ID失败: {}", e))?;

            let open_tabs_data: Vec<serde_json::Value> = config
                .open_tabs
                .iter()
                .zip(config.open_tab_types.iter())
                .map(|(id, ttype)| serde_json::json!({"id": id, "type": ttype}))
                .collect();
            let open_tabs_json = serde_json::to_string(&open_tabs_data)
                .map_err(|e| repo_error!("序列化标签页失败: {}", e))?;

            let request_tabs_json = serde_json::to_string(&config.request_tabs)
                .map_err(|e| repo_error!("序列化请求标签页失败: {}", e))?;

            conn.execute(
                "INSERT INTO app_state \
                 (id, expanded_ids_json, open_tabs_json, active_tab_index, request_tabs_json) \
                 VALUES (1, ?1, ?2, ?3, ?4) \
                 ON CONFLICT(id) DO UPDATE SET \
                 expanded_ids_json = excluded.expanded_ids_json, \
                 open_tabs_json = excluded.open_tabs_json, \
                 active_tab_index = excluded.active_tab_index, \
                 request_tabs_json = excluded.request_tabs_json",
                params![
                    expanded_ids_json,
                    open_tabs_json,
                    config.active_tab_index as i32,
                    request_tabs_json,
                ],
            )
            .map_err(|e| repo_error!("写入记忆数据失败: {}", e))?;

            Ok(())
        })
    }

    fn read_cookies(&self, workspace_id: &str) -> Result<CookiesConfig, String> {
        let cookies = self.get_all_cookies(workspace_id)?;
        Ok(CookiesConfig { cookies })
    }

    fn write_cookies(&self, workspace_id: &str, config: &CookiesConfig) -> Result<(), String> {
        for cookie in &config.cookies {
            if cookie.name.is_empty() {
                return Err(repo_error!("Cookie 名称不能为空"));
            }
            if cookie.domain.is_empty() {
                return Err(repo_error!("Cookie domain 不能为空"));
            }
        }

        let ws = workspace_id.to_string();
        with_transaction(&ws, |conn| {
            conn.execute("DELETE FROM cookies", [])
                .map_err(|e| repo_error!("清除 Cookie 失败: {}", e))?;

            for cookie in &config.cookies {
                conn.execute(
                    "INSERT INTO cookies \
                     (name, domain, path, value, expires, max_age, secure, http_only, created_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        cookie.name,
                        cookie.domain,
                        cookie.path,
                        cookie.value,
                        cookie.expires,
                        cookie.max_age,
                        cookie.secure as i32,
                        cookie.http_only as i32,
                        cookie.created_at,
                    ],
                )
                .map_err(|e| repo_error!("插入 Cookie 失败: {}", e))?;
            }

            Ok(())
        })
    }

    fn get_all_cookies(&self, workspace_id: &str) -> Result<Vec<Cookie>, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT name, domain, path, value, expires, max_age, secure, http_only, created_at FROM cookies",
                )
                .map_err(|e| repo_error!("准备查询Cookie失败: {}", e))?;

            let rows: Vec<Cookie> = stmt
                .query_map([], |row| {
                    Ok(Cookie {
                        name: row.get(0)?,
                        domain: row.get(1)?,
                        path: row.get(2)?,
                        value: row.get(3)?,
                        expires: row.get(4)?,
                        max_age: row.get(5)?,
                        secure: row.get::<_, i32>(6)? != 0,
                        http_only: row.get::<_, i32>(7)? != 0,
                        created_at: row.get(8)?,
                    })
                })
                .map_err(|e| repo_error!("查询Cookie失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析Cookie行数据失败: {}", e))?;

            Ok(rows)
        })
    }

    fn add_or_update_cookie(&self, workspace_id: &str, cookie: &Cookie) -> Result<(), String> {
        if cookie.name.is_empty() {
            return Err(repo_error!("Cookie 名称不能为空"));
        }
        if cookie.domain.is_empty() {
            return Err(repo_error!("Cookie domain 不能为空"));
        }

        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute(
                "INSERT OR REPLACE INTO cookies \
                 (name, domain, path, value, expires, max_age, secure, http_only, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    cookie.name,
                    cookie.domain,
                    cookie.path,
                    cookie.value,
                    cookie.expires,
                    cookie.max_age,
                    cookie.secure as i32,
                    cookie.http_only as i32,
                    cookie.created_at,
                ],
            )
            .map_err(|e| repo_error!("添加/更新 Cookie 失败: {}", e))?;

            Ok(())
        })
    }

    fn delete_cookie(&self, workspace_id: &str, name: &str, domain: &str) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute(
                "DELETE FROM cookies WHERE name = ?1 AND domain = ?2",
                params![name, domain],
            )
            .map_err(|e| repo_error!("删除 Cookie 失败: {}", e))?;

            Ok(())
        })
    }

    fn clear_cookies(&self, workspace_id: &str) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute("DELETE FROM cookies", [])
                .map_err(|e| repo_error!("清除所有 Cookie 失败: {}", e))?;

            Ok(())
        })
    }
}
