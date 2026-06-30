//! 工作区导入导出应用服务

use crate::domain::models::{
    AppStateExport, AssertionExport, ChatSession, CollectionExportItem, Cookie, DocExportItem,
    Environment, FileInfoExport, FormFieldExport, HeaderExport, HistoryEntry, OpenTabExport,
    Orchestration, OrchestrationRun, ParamExport, SavedResponse, ScriptExportItem,
    StressConfigExport, StressTestResult, VariableExport, Workspace, WorkspaceExport,
    WorkspaceExportData, WorkspaceExportMeta, WorkspaceExportStats, WorkspaceImportPreview,
    WsConfigItem,
};
use crate::domain::repositories::AppConfigRepository;
use crate::infrastructure::sqlite::connection::with_connection;
use crate::infrastructure::RepositoryFactory;
use crate::repo_error;
use rusqlite::{params, Connection};
use std::collections::HashMap;

pub struct WorkspaceIOService {
    app_config_repo: Box<dyn AppConfigRepository>,
}

impl WorkspaceIOService {
    pub fn new() -> Self {
        Self {
            app_config_repo: RepositoryFactory::get_app_config_repository(),
        }
    }
}

impl Default for WorkspaceIOService {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceIOService {
    pub fn export_workspace(&self, workspace_id: &str) -> Result<String, String> {
        let config = self.app_config_repo.read()?;
        let workspace = config
            .workspaces
            .iter()
            .find(|w| w.id == workspace_id)
            .ok_or_else(|| repo_error!("工作区不存在: {}", workspace_id))?;

        let data = Self::read_all_workspace_data(workspace_id)?;

        let export = WorkspaceExport {
            version: WorkspaceExport::CURRENT_VERSION.to_string(),
            exported_at: chrono::Local::now().to_rfc3339(),
            app_version: Self::get_app_version(),
            workspace: WorkspaceExportMeta {
                name: workspace.name.clone(),
                description: workspace.description.clone(),
            },
            data,
        };

        serde_json::to_string_pretty(&export).map_err(|e| repo_error!("序列化导出数据失败: {}", e))
    }

    pub fn preview_import(content: &str) -> Result<WorkspaceImportPreview, String> {
        let export: WorkspaceExport =
            serde_json::from_str(content).map_err(|e| repo_error!("解析导入文件失败: {}", e))?;

        if export.version != WorkspaceExport::CURRENT_VERSION {
            return Err(repo_error!(
                "不支持的导出文件版本: {}（当前支持版本: {}）",
                export.version,
                WorkspaceExport::CURRENT_VERSION
            ));
        }

        let stats = Self::calculate_stats(&export.data);

        Ok(WorkspaceImportPreview {
            name: export.workspace.name,
            description: export.workspace.description,
            exported_at: export.exported_at,
            app_version: export.app_version,
            stats,
        })
    }

    pub fn import_workspace(
        &self,
        content: &str,
        new_name: Option<String>,
    ) -> Result<Workspace, String> {
        let export: WorkspaceExport =
            serde_json::from_str(content).map_err(|e| repo_error!("解析导入文件失败: {}", e))?;

        if export.version != WorkspaceExport::CURRENT_VERSION {
            return Err(repo_error!("不支持的导出文件版本: {}", export.version));
        }

        let config = self.app_config_repo.read()?;
        let existing_names: Vec<&str> = config.workspaces.iter().map(|w| w.name.as_str()).collect();

        let final_name = new_name
            .unwrap_or_else(|| Self::generate_unique_name(&export.workspace.name, &existing_names));

        let new_workspace_id = crate::domain::models::common::generate_id("ws");
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let new_workspace = Workspace {
            id: new_workspace_id.clone(),
            name: final_name,
            description: export.workspace.description,
            created_at: now.clone(),
            last_opened: now,
            last_api_id: None,
            last_backup_at: None,
        };

        Self::write_all_workspace_data(&new_workspace_id, &export.data)?;

        let mut config = self.app_config_repo.read()?;
        config.workspaces.push(new_workspace.clone());
        config.last_workspace_id = Some(new_workspace_id.clone());
        self.app_config_repo.write(&config)?;

        Ok(new_workspace)
    }

    /// 将备份内容恢复到已有工作区（覆盖该工作区的全部数据，保留 id 与名称）
    pub fn restore_into_workspace(
        &self,
        workspace_id: &str,
        content: &str,
    ) -> Result<Workspace, String> {
        let export: WorkspaceExport =
            serde_json::from_str(content).map_err(|e| repo_error!("解析导入文件失败: {}", e))?;

        if export.version != WorkspaceExport::CURRENT_VERSION {
            return Err(repo_error!("不支持的导出文件版本: {}", export.version));
        }

        // 重置目标工作区 schema（DROP 所有表并按 SCHEMA_SQL 重建，得到空表）
        crate::infrastructure::sqlite::connection::reset_workspace_schema(workspace_id)?;
        // 写入备份数据
        Self::write_all_workspace_data(workspace_id, &export.data)?;

        // 返回当前工作区（id 与名称保持不变）
        let config = self.app_config_repo.read()?;
        let workspace = config
            .workspaces
            .iter()
            .find(|w| w.id == workspace_id)
            .cloned()
            .ok_or_else(|| repo_error!("工作区不存在: {}", workspace_id))?;
        Ok(workspace)
    }

    fn get_app_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn generate_unique_name(base_name: &str, existing_names: &[&str]) -> String {
        if !existing_names.contains(&base_name) {
            return base_name.to_string();
        }

        let mut counter = 1;
        loop {
            let new_name = format!("{} ({})", base_name, counter);
            if !existing_names.contains(&new_name.as_str()) {
                return new_name;
            }
            counter += 1;
        }
    }

    fn read_all_workspace_data(workspace_id: &str) -> Result<WorkspaceExportData, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let environments = Self::read_environments(conn)?;
            let active_environment_id = Self::read_active_environment_id(conn)?;
            let collection_items = Self::read_collection_items(conn)?;
            let scripts = Self::read_scripts(conn)?;
            let saved_responses = Self::read_saved_responses(conn)?;
            let cookies = Self::read_cookies(conn)?;
            let history_entries = Self::read_history_entries(conn)?;
            let orchestrations = Self::read_orchestrations(conn)?;
            let orchestration_runs = Self::read_orchestration_runs(conn)?;
            let stress_configs = Self::read_stress_configs(conn)?;
            let stress_results = Self::read_stress_results(conn)?;
            let docs = Self::read_docs(conn)?;
            let chat_sessions = Self::read_chat_sessions(conn)?;
            let app_state = Self::read_app_state(conn)?;
            let ws_configs = Self::read_ws_configs(conn)?;

            Ok(WorkspaceExportData {
                environments,
                active_environment_id,
                collection_items,
                scripts,
                saved_responses,
                cookies,
                history_entries,
                orchestrations,
                orchestration_runs,
                stress_configs,
                stress_results,
                docs,
                chat_sessions,
                app_state,
                ws_configs,
            })
        })
    }

    fn write_all_workspace_data(
        workspace_id: &str,
        data: &WorkspaceExportData,
    ) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute_batch("BEGIN")
                .map_err(|e| repo_error!("开始事务失败: {}", e))?;

            Self::write_environments(conn, &data.environments)?;
            if let Some(app_state) = &data.app_state {
                Self::write_app_state(conn, app_state, &data.active_environment_id)?;
            }
            Self::write_collection_items(conn, &data.collection_items)?;
            Self::write_scripts(conn, &data.scripts)?;
            Self::write_saved_responses(conn, &data.saved_responses)?;
            Self::write_cookies(conn, &data.cookies)?;
            Self::write_history_entries(conn, &data.history_entries)?;
            Self::write_orchestrations(conn, &data.orchestrations)?;
            Self::write_orchestration_runs(conn, &data.orchestration_runs)?;
            Self::write_stress_configs(conn, &data.stress_configs)?;
            Self::write_stress_results(conn, &data.stress_results)?;
            Self::write_docs(conn, &data.docs)?;
            Self::write_chat_sessions(conn, &data.chat_sessions)?;
            Self::write_ws_configs(conn, &data.ws_configs)?;

            conn.execute_batch("COMMIT")
                .map_err(|e| repo_error!("提交事务失败: {}", e))?;

            Ok(())
        })
    }

    fn calculate_stats(data: &WorkspaceExportData) -> WorkspaceExportStats {
        let collections = data
            .collection_items
            .iter()
            .filter(|i| i.item_type == "collection")
            .count();
        let apis = data
            .collection_items
            .iter()
            .filter(|i| i.item_type == "api")
            .count();
        let websockets = data
            .collection_items
            .iter()
            .filter(|i| i.item_type == "websocket")
            .count();

        WorkspaceExportStats {
            environments: data.environments.len(),
            collections,
            apis,
            websockets,
            scripts: data.scripts.len(),
            saved_responses: data.saved_responses.len(),
            cookies: data.cookies.len(),
            history_entries: data.history_entries.len(),
            orchestrations: data.orchestrations.len(),
            stress_results: data.stress_results.len(),
            docs: data.docs.len(),
            chat_sessions: data.chat_sessions.len(),
            ws_configs: data.ws_configs.len(),
        }
    }

    fn read_environments(conn: &Connection) -> Result<Vec<Environment>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, variables_json, common_headers_json FROM environments ORDER BY order_index",
            )
            .map_err(|e| repo_error!("准备查询环境失败: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| repo_error!("查询环境失败: {}", e))?;

        let mut environments = Vec::new();
        for (id, name, variables_json, common_headers_json) in rows.flatten() {
            let variables: Vec<crate::domain::models::Variable> =
                serde_json::from_str(&variables_json)
                    .map_err(|e| format!("反序列化变量失败: {}", e))?;
            let common_headers: Option<Vec<crate::domain::models::Header>> =
                serde_json::from_str(&common_headers_json)
                    .map(
                        |h: Vec<crate::domain::models::Header>| {
                            if h.is_empty() {
                                None
                            } else {
                                Some(h)
                            }
                        },
                    )
                    .map_err(|e| format!("反序列化公共请求头失败: {}", e))?;

            environments.push(Environment {
                id,
                name,
                variables,
                common_headers,
            });
        }

        Ok(environments)
    }

    fn read_active_environment_id(conn: &Connection) -> Result<Option<String>, String> {
        let result = conn.query_row(
            "SELECT active_environment_id FROM app_state WHERE id = 1",
            [],
            |row| row.get::<_, Option<String>>(0),
        );
        Ok(result.ok().flatten())
    }

    fn read_collection_items(conn: &Connection) -> Result<Vec<CollectionExportItem>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, item_type, parent_id, order_index, \
                 method, url, body, body_type, \
                 params_json, headers_json, form_fields_json, form_files_json, \
                 common_headers_json, variables_json \
                 FROM collection_items ORDER BY parent_id, order_index",
            )
            .map_err(|e| repo_error!("准备查询集合项失败: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, i32>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, String>(12)?,
                    row.get::<_, String>(13)?,
                    row.get::<_, String>(14)?,
                    row.get::<_, String>(15)?,
                ))
            })
            .map_err(|e| repo_error!("查询集合项失败: {}", e))?;

        let mut items = Vec::new();
        for (
            id,
            name,
            description,
            item_type,
            parent_id,
            order_index,
            method,
            url,
            body,
            body_type,
            params_json,
            headers_json,
            form_fields_json,
            form_files_json,
            common_headers_json,
            variables_json,
        ) in rows.flatten()
        {
            let params: Vec<ParamExport> = serde_json::from_str(&params_json)
                .map_err(|e| format!("反序列化参数失败: {}", e))?;
            let headers: Vec<HeaderExport> = serde_json::from_str(&headers_json)
                .map_err(|e| format!("反序列化请求头失败: {}", e))?;
            let form_fields: Vec<crate::domain::models::FormField> =
                serde_json::from_str(&form_fields_json)
                    .map_err(|e| format!("反序列化表单字段失败: {}", e))?;
            let _form_files: HashMap<String, Vec<crate::domain::models::FileInfo>> =
                serde_json::from_str(&form_files_json)
                    .map_err(|e| format!("反序列化表单文件失败: {}", e))?;
            let common_headers: Vec<HeaderExport> = serde_json::from_str(&common_headers_json)
                .map_err(|e| format!("反序列化公共请求头失败: {}", e))?;
            let variables: Vec<VariableExport> = serde_json::from_str(&variables_json)
                .map_err(|e| format!("反序列化集合变量失败: {}", e))?;

            let form_fields_export: Vec<FormFieldExport> = form_fields
                .into_iter()
                .map(|f| FormFieldExport {
                    key: f.key,
                    value: f.value,
                    field_type: f.field_type,
                    enabled: f.enabled,
                    files: f.files.map(|files| {
                        files
                            .into_iter()
                            .map(|fi| FileInfoExport {
                                path: fi.path,
                                name: fi.name,
                            })
                            .collect()
                    }),
                })
                .collect();

            items.push(CollectionExportItem {
                id,
                name,
                description,
                item_type,
                parent_id,
                order_index,
                method,
                url,
                body,
                body_type,
                params,
                headers,
                form_fields: form_fields_export,
                common_headers,
                variables,
                ws_config: None,
            });
        }

        Ok(items)
    }

    fn read_scripts(conn: &Connection) -> Result<Vec<ScriptExportItem>, String> {
        let mut stmt = conn
            .prepare("SELECT target_type, target_id, script_kind, filename, content FROM scripts")
            .map_err(|e| repo_error!("准备查询脚本失败: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(|e| repo_error!("查询脚本失败: {}", e))?;

        let scripts = rows
            .filter_map(|r| r.ok())
            .map(
                |(target_type, target_id, script_kind, filename, content)| ScriptExportItem {
                    target_type,
                    target_id,
                    script_kind,
                    filename,
                    content,
                },
            )
            .collect();

        Ok(scripts)
    }

    fn read_saved_responses(conn: &Connection) -> Result<Vec<SavedResponse>, String> {
        let mut stmt = conn
            .prepare("SELECT id, name, created_at, api_id, doc_content FROM saved_responses")
            .map_err(|e| repo_error!("准备查询保存响应失败: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(SavedResponse {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    api_id: row.get(3)?,
                    doc_content: row.get(4)?,
                })
            })
            .map_err(|e| repo_error!("查询保存响应失败: {}", e))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn read_cookies(conn: &Connection) -> Result<Vec<Cookie>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT name, domain, path, value, expires, max_age, secure, http_only, created_at FROM cookies",
            )
            .map_err(|e| repo_error!("准备查询 Cookie 失败: {}", e))?;

        let rows = stmt
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
            .map_err(|e| repo_error!("查询 Cookie 失败: {}", e))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn read_history_entries(conn: &Connection) -> Result<Vec<HistoryEntry>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, method, url, resolved_url, status, status_text, \
                 response_body, time, size, created_at, body, body_type, \
                 api_id, api_name, date, \
                 request_headers_json, response_headers_json, form_fields_json \
                 FROM history_entries",
            )
            .map_err(|e| repo_error!("准备查询历史记录失败: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
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
                    row.get::<_, String>(17)?,
                ))
            })
            .map_err(|e| repo_error!("查询历史记录失败: {}", e))?;

        let rows: Vec<_> = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| repo_error!("解析历史记录行失败: {}", e))?;

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
                    _date,
                    request_headers_json,
                    response_headers_json,
                    form_fields_json,
                )| {
                    let headers: Vec<crate::domain::models::Header> =
                        serde_json::from_str(&request_headers_json)
                            .map_err(|e| format!("反序列化请求头失败: {}", e))?;
                    let response_headers: HashMap<String, String> =
                        serde_json::from_str(&response_headers_json)
                            .map_err(|e| format!("反序列化响应头失败: {}", e))?;
                    let form_fields: Vec<crate::domain::models::FormField> =
                        serde_json::from_str(&form_fields_json)
                            .map_err(|e| format!("反序列化表单字段失败: {}", e))?;

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
                },
            )
            .collect::<Result<Vec<_>, String>>()?;

        Ok(entries)
    }

    fn read_orchestrations(conn: &Connection) -> Result<Vec<Orchestration>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, created_at, updated_at, steps_json, schedule_json \
                 FROM orchestrations ORDER BY order_index",
            )
            .map_err(|e| repo_error!("准备查询编排失败: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(|e| repo_error!("查询编排失败: {}", e))?;

        let rows: Vec<_> = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| repo_error!("解析编排行失败: {}", e))?;

        let orchestrations: Vec<Orchestration> = rows
            .into_iter()
            .map(
                |(id, name, description, created_at, updated_at, steps_json, schedule_json)| {
                    let steps: Vec<crate::domain::models::OrchestrationStep> =
                        serde_json::from_str(&steps_json)
                            .map_err(|e| format!("反序列化编排步骤失败: {}", e))?;
                    let schedule: Option<crate::domain::models::OrchestrationSchedule> =
                        serde_json::from_str(&schedule_json)
                            .map(|s: crate::domain::models::OrchestrationSchedule| Some(s))
                            .ok()
                            .flatten();

                    Ok(Orchestration {
                        id,
                        name,
                        description,
                        created_at,
                        updated_at,
                        steps,
                        schedule,
                    })
                },
            )
            .collect::<Result<Vec<_>, String>>()?;

        Ok(orchestrations)
    }

    fn read_orchestration_runs(conn: &Connection) -> Result<Vec<OrchestrationRun>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, orchestration_id, status, start_time, end_time, \
                 total_time, success_count, failed_count, skipped_count, steps_json \
                 FROM orchestration_runs",
            )
            .map_err(|e| repo_error!("准备查询编排执行记录失败: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, i64>(8)?,
                    row.get::<_, String>(9)?,
                ))
            })
            .map_err(|e| repo_error!("查询编排执行记录失败: {}", e))?;

        let rows_vec: Vec<_> = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| repo_error!("解析编排执行行失败: {}", e))?;

        let runs: Vec<OrchestrationRun> = rows_vec
            .into_iter()
            .map(
                |(
                    id,
                    orchestration_id,
                    status,
                    start_time,
                    end_time,
                    total_time,
                    success_count,
                    failed_count,
                    skipped_count,
                    steps_json,
                )| {
                    let steps: Vec<crate::domain::models::StepRunResult> =
                        serde_json::from_str(&steps_json)
                            .map_err(|e| format!("反序列化步骤结果失败: {}", e))?;

                    Ok(OrchestrationRun {
                        id,
                        orchestration_id,
                        status,
                        start_time,
                        end_time,
                        total_time: total_time as u64,
                        success_count: success_count as usize,
                        failed_count: failed_count as usize,
                        skipped_count: skipped_count as usize,
                        steps,
                    })
                },
            )
            .collect::<Result<Vec<_>, String>>()?;

        Ok(runs)
    }

    fn read_stress_configs(conn: &Connection) -> Result<Vec<StressConfigExport>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT api_id, concurrent, total_requests, duration_seconds, \
                 ramp_up_seconds, timeout_ms, assertions_json FROM stress_configs",
            )
            .map_err(|e| repo_error!("准备查询压测配置失败: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<i32>>(3)?,
                    row.get::<_, i32>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(|e| repo_error!("查询压测配置失败: {}", e))?;

        let configs: Vec<StressConfigExport> = rows
            .filter_map(|r| r.ok())
            .map(
                |(
                    api_id,
                    concurrent,
                    total_requests,
                    duration_seconds,
                    ramp_up_seconds,
                    timeout_ms,
                    assertions_json,
                )| {
                    let assertions: Vec<AssertionExport> =
                        serde_json::from_str(&assertions_json).unwrap_or_default();
                    Ok(StressConfigExport {
                        api_id,
                        concurrent: concurrent as u32,
                        total_requests: total_requests.map(|v| v as u64),
                        duration_seconds: duration_seconds.map(|v| v as u32),
                        ramp_up_seconds: ramp_up_seconds as u32,
                        timeout_ms: timeout_ms as u64,
                        assertions,
                    })
                },
            )
            .collect::<Result<Vec<StressConfigExport>, String>>()?;

        Ok(configs)
    }

    fn read_stress_results(conn: &Connection) -> Result<Vec<StressTestResult>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, api_id, config_json, start_time, end_time, \
                 total_requests, successful_requests, failed_requests, \
                 total_time_ms, qps, avg_time_ms, min_time_ms, max_time_ms, \
                 p50_time_ms, p90_time_ms, p95_time_ms, p99_time_ms, \
                 success_rate, status_distribution_json, error_distribution_json \
                 FROM stress_results",
            )
            .map_err(|e| repo_error!("准备查询压测结果失败: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, i64>(8)?,
                    row.get::<_, f64>(9)?,
                    row.get::<_, f64>(10)?,
                    row.get::<_, i64>(11)?,
                    row.get::<_, i64>(12)?,
                    row.get::<_, f64>(13)?,
                    row.get::<_, f64>(14)?,
                    row.get::<_, f64>(15)?,
                    row.get::<_, f64>(16)?,
                    row.get::<_, f64>(17)?,
                    row.get::<_, String>(18)?,
                    row.get::<_, String>(19)?,
                ))
            })
            .map_err(|e| repo_error!("查询压测结果失败: {}", e))?;

        let results: Vec<StressTestResult> = rows
            .filter_map(|r| r.ok())
            .map(
                |(
                    id,
                    api_id,
                    config_json,
                    start_time,
                    end_time,
                    total_requests,
                    successful_requests,
                    failed_requests,
                    total_time_ms,
                    qps,
                    avg_time_ms,
                    min_time_ms,
                    max_time_ms,
                    p50_time_ms,
                    p90_time_ms,
                    p95_time_ms,
                    p99_time_ms,
                    success_rate,
                    status_distribution_json,
                    error_distribution_json,
                )| {
                    let config: crate::domain::models::StressTestConfig =
                        serde_json::from_str(&config_json)
                            .map_err(|e| format!("反序列化压测配置失败: {}", e))?;
                    let status_distribution: HashMap<String, u64> =
                        serde_json::from_str(&status_distribution_json)
                            .map_err(|e| format!("反序列化状态分布失败: {}", e))?;
                    let error_distribution: HashMap<String, u64> =
                        serde_json::from_str(&error_distribution_json)
                            .map_err(|e| format!("反序列化错误分布失败: {}", e))?;

                    let failed_request_details = Self::read_stress_result_details(conn, &id)?;
                    let history = Self::read_stress_result_points(conn, &id)?;

                    Ok(StressTestResult {
                        id,
                        api_id,
                        config,
                        start_time,
                        end_time,
                        total_requests: total_requests as u64,
                        successful_requests: successful_requests as u64,
                        failed_requests: failed_requests as u64,
                        total_time_ms: total_time_ms as u64,
                        qps,
                        avg_time_ms,
                        min_time_ms: min_time_ms as u64,
                        max_time_ms: max_time_ms as u64,
                        p50_time_ms,
                        p90_time_ms,
                        p95_time_ms,
                        p99_time_ms,
                        success_rate,
                        status_distribution,
                        error_distribution,
                        failed_request_details,
                        history,
                    })
                },
            )
            .collect::<Result<Vec<StressTestResult>, String>>()?;

        Ok(results)
    }

    fn read_stress_result_details(
        conn: &Connection,
        result_id: &str,
    ) -> Result<Vec<crate::domain::models::FailedRequest>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT time, error, status, elapsed_ms \
                 FROM stress_result_details WHERE result_id = ?1 ORDER BY rowid",
            )
            .map_err(|e| repo_error!("准备查询压测失败详情失败: {}", e))?;

        let rows = stmt
            .query_map(params![result_id], |row| {
                Ok(crate::domain::models::FailedRequest {
                    time: row.get(0)?,
                    error: row.get(1)?,
                    status: row.get::<_, Option<i32>>(2)?.map(|v| v as u16),
                    elapsed_ms: row.get::<_, i64>(3)? as u64,
                })
            })
            .map_err(|e| repo_error!("查询压测失败详情失败: {}", e))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn read_stress_result_points(
        conn: &Connection,
        result_id: &str,
    ) -> Result<Vec<crate::domain::models::HistoryPoint>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT second, qps, avg_time_ms, successful, failed, requests, concurrent \
                 FROM stress_result_points WHERE result_id = ?1 ORDER BY second",
            )
            .map_err(|e| repo_error!("准备查询压测历史数据点失败: {}", e))?;

        let rows = stmt
            .query_map(params![result_id], |row| {
                Ok(crate::domain::models::HistoryPoint {
                    second: row.get::<_, i32>(0)? as u32,
                    qps: row.get(1)?,
                    avg_time_ms: row.get(2)?,
                    successful: row.get::<_, i64>(3)? as u64,
                    failed: row.get::<_, i64>(4)? as u64,
                    requests: row.get::<_, i64>(5)? as u64,
                    concurrent: row.get::<_, i32>(6)? as u32,
                })
            })
            .map_err(|e| repo_error!("查询压测历史数据点失败: {}", e))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn read_docs(conn: &Connection) -> Result<Vec<DocExportItem>, String> {
        let mut stmt = conn
            .prepare("SELECT api_id, updated_at, content FROM docs")
            .map_err(|e| repo_error!("准备查询文档失败: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(DocExportItem {
                    api_id: row.get(0)?,
                    updated_at: row.get(1)?,
                    content: row.get(2)?,
                })
            })
            .map_err(|e| repo_error!("查询文档失败: {}", e))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn read_ws_configs(conn: &Connection) -> Result<Vec<WsConfigItem>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, url, headers_json, params_json, created_at, updated_at \
                 FROM ws_configs ORDER BY order_index",
            )
            .map_err(|e| repo_error!("准备查询 WebSocket 配置失败: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(|e| repo_error!("查询 WebSocket 配置失败: {}", e))?;

        let configs: Vec<WsConfigItem> = rows
            .filter_map(|r| r.ok())
            .map(
                |(id, name, url, headers_json, params_json, created_at, updated_at)| {
                    let headers: Vec<HeaderExport> =
                        serde_json::from_str(&headers_json).unwrap_or_default();
                    let params: Vec<ParamExport> =
                        serde_json::from_str(&params_json).unwrap_or_default();
                    Ok(WsConfigItem {
                        id,
                        name,
                        url,
                        headers,
                        params,
                        created_at,
                        updated_at,
                    })
                },
            )
            .collect::<Result<Vec<WsConfigItem>, String>>()?;

        Ok(configs)
    }

    fn read_chat_sessions(conn: &Connection) -> Result<Vec<ChatSession>, String> {
        let mut stmt = conn
            .prepare("SELECT id, created_at, title FROM chat_sessions ORDER BY created_at")
            .map_err(|e| repo_error!("准备查询聊天会话失败: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|e| repo_error!("查询聊天会话失败: {}", e))?;

        let mut sessions = Vec::new();
        for (id, created_at, title) in rows.flatten() {
            let messages = Self::read_chat_messages(conn, &id)?;
            sessions.push(ChatSession {
                id,
                created_at,
                messages,
                title,
            });
        }

        Ok(sessions)
    }

    fn read_chat_messages(
        conn: &Connection,
        session_id: &str,
    ) -> Result<Vec<crate::domain::models::ChatMessage>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT role, content, reasoning, timestamp \
                 FROM chat_messages WHERE session_id = ?1 ORDER BY rowid",
            )
            .map_err(|e| repo_error!("准备查询聊天消息失败: {}", e))?;

        let rows = stmt
            .query_map(params![session_id], |row| {
                Ok(crate::domain::models::ChatMessage {
                    role: row.get(0)?,
                    content: row.get(1)?,
                    reasoning: row.get(2)?,
                    timestamp: row.get(3)?,
                })
            })
            .map_err(|e| repo_error!("查询聊天消息失败: {}", e))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn read_app_state(conn: &Connection) -> Result<Option<AppStateExport>, String> {
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
                    .map_err(|e| format!("反序列化展开ID失败: {}", e))?;
                let open_tabs_data: Vec<serde_json::Value> = serde_json::from_str(&open_tabs_json)
                    .map_err(|e| format!("反序列化标签页失败: {}", e))?;
                let open_tabs: Vec<OpenTabExport> = open_tabs_data
                    .into_iter()
                    .filter_map(|item| {
                        if let (Some(id), Some(ttype)) = (
                            item.get("id").and_then(|v| v.as_str()),
                            item.get("type").and_then(|v| v.as_str()),
                        ) {
                            Some(OpenTabExport {
                                id: id.to_string(),
                                tab_type: ttype.to_string(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                let request_tabs: HashMap<String, String> =
                    serde_json::from_str(&request_tabs_json)
                        .map_err(|e| format!("反序列化请求标签页失败: {}", e))?;

                Ok(Some(AppStateExport {
                    expanded_ids,
                    open_tabs,
                    active_tab_index: active_tab_index as usize,
                    request_tabs,
                }))
            }
            Err(_) => Ok(None),
        }
    }

    fn write_environments(conn: &Connection, environments: &[Environment]) -> Result<(), String> {
        conn.execute("DELETE FROM environments", [])
            .map_err(|e| repo_error!("清除环境失败: {}", e))?;

        for (i, env) in environments.iter().enumerate() {
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

        Ok(())
    }

    fn write_collection_items(
        conn: &Connection,
        items: &[CollectionExportItem],
    ) -> Result<(), String> {
        conn.execute("DELETE FROM collection_items", [])
            .map_err(|e| repo_error!("清除集合项失败: {}", e))?;

        for item in items {
            let params_json = serde_json::to_string(&item.params)
                .map_err(|e| repo_error!("序列化参数失败: {}", e))?;
            let headers_json = serde_json::to_string(&item.headers)
                .map_err(|e| repo_error!("序列化请求头失败: {}", e))?;

            let form_fields: Vec<crate::domain::models::FormField> = item
                .form_fields
                .iter()
                .map(|f| crate::domain::models::FormField {
                    key: f.key.clone(),
                    value: f.value.clone(),
                    field_type: f.field_type.clone(),
                    enabled: f.enabled,
                    files: f.files.clone().map(|files| {
                        files
                            .into_iter()
                            .map(|fi| crate::domain::models::FileInfo {
                                path: fi.path,
                                name: fi.name,
                            })
                            .collect()
                    }),
                })
                .collect();
            let form_fields_json = serde_json::to_string(&form_fields)
                .map_err(|e| repo_error!("序列化表单字段失败: {}", e))?;

            let form_files_map: HashMap<String, Vec<crate::domain::models::FileInfo>> = item
                .form_fields
                .iter()
                .filter_map(|f| {
                    f.files.as_ref().map(|files| {
                        (
                            f.key.clone(),
                            files
                                .iter()
                                .map(|fi| crate::domain::models::FileInfo {
                                    path: fi.path.clone(),
                                    name: fi.name.clone(),
                                })
                                .collect(),
                        )
                    })
                })
                .collect();
            let form_files_json = serde_json::to_string(&form_files_map)
                .map_err(|e| repo_error!("序列化表单文件失败: {}", e))?;

            let common_headers_json = serde_json::to_string(&item.common_headers)
                .map_err(|e| repo_error!("序列化公共请求头失败: {}", e))?;
            let variables_json = serde_json::to_string(&item.variables)
                .map_err(|e| repo_error!("序列化集合变量失败: {}", e))?;

            conn.execute(
                "INSERT INTO collection_items \
                 (id, name, description, item_type, parent_id, order_index, \
                  method, url, body, body_type, \
                  params_json, headers_json, form_fields_json, form_files_json, \
                  common_headers_json, variables_json, saved_response_ids_json) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, '[]')",
                params![
                    item.id,
                    item.name,
                    item.description,
                    item.item_type,
                    item.parent_id,
                    item.order_index,
                    item.method,
                    item.url,
                    item.body,
                    item.body_type,
                    params_json,
                    headers_json,
                    form_fields_json,
                    form_files_json,
                    common_headers_json,
                    variables_json,
                ],
            )
            .map_err(|e| repo_error!("插入集合项失败: {}", e))?;
        }

        Ok(())
    }

    fn write_scripts(conn: &Connection, scripts: &[ScriptExportItem]) -> Result<(), String> {
        conn.execute("DELETE FROM scripts", [])
            .map_err(|e| repo_error!("清除脚本失败: {}", e))?;

        for script in scripts {
            let id = match &script.target_id {
                Some(tid) => format!(
                    "script_{}_{}_{}",
                    script.target_type, tid, script.script_kind
                ),
                None => format!("script_{}_{}", script.target_type, script.script_kind),
            };

            conn.execute(
                "INSERT INTO scripts (id, target_type, target_id, script_kind, filename, content) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    id,
                    script.target_type,
                    script.target_id,
                    script.script_kind,
                    script.filename,
                    script.content
                ],
            )
            .map_err(|e| repo_error!("插入脚本失败: {}", e))?;
        }

        Ok(())
    }

    fn write_saved_responses(conn: &Connection, responses: &[SavedResponse]) -> Result<(), String> {
        conn.execute("DELETE FROM saved_responses", [])
            .map_err(|e| repo_error!("清除保存响应失败: {}", e))?;

        for resp in responses {
            conn.execute(
                "INSERT INTO saved_responses (id, name, created_at, api_id, doc_content) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    resp.id,
                    resp.name,
                    resp.created_at,
                    resp.api_id,
                    resp.doc_content
                ],
            )
            .map_err(|e| repo_error!("插入保存响应失败: {}", e))?;
        }

        Ok(())
    }

    fn write_cookies(conn: &Connection, cookies: &[Cookie]) -> Result<(), String> {
        conn.execute("DELETE FROM cookies", [])
            .map_err(|e| repo_error!("清除 Cookie 失败: {}", e))?;

        for cookie in cookies {
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
    }

    fn write_history_entries(conn: &Connection, entries: &[HistoryEntry]) -> Result<(), String> {
        conn.execute("DELETE FROM history_entries", [])
            .map_err(|e| repo_error!("清除历史记录失败: {}", e))?;

        for entry in entries {
            let date = Self::extract_date_from_timestamp(&entry.created_at);

            let request_headers_json = serde_json::to_string(&entry.headers)
                .map_err(|e| repo_error!("序列化请求头失败: {}", e))?;
            let response_headers_json = serde_json::to_string(&entry.response_headers)
                .map_err(|e| repo_error!("序列化响应头失败: {}", e))?;
            let form_fields_json =
                serde_json::to_string(entry.form_fields.as_ref().unwrap_or(&Vec::new()))
                    .map_err(|e| repo_error!("序列化表单字段失败: {}", e))?;

            conn.execute(
                "INSERT INTO history_entries \
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
        }

        Ok(())
    }

    fn extract_date_from_timestamp(timestamp: &str) -> String {
        use chrono::{DateTime, Local};

        if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
            dt.with_timezone(&Local).format("%Y-%m-%d").to_string()
        } else {
            Local::now().format("%Y-%m-%d").to_string()
        }
    }

    fn write_orchestrations(conn: &Connection, items: &[Orchestration]) -> Result<(), String> {
        conn.execute("DELETE FROM orchestrations", [])
            .map_err(|e| repo_error!("清除编排失败: {}", e))?;

        for (i, orch) in items.iter().enumerate() {
            let steps_json = serde_json::to_string(&orch.steps)
                .map_err(|e| repo_error!("序列化编排步骤失败: {}", e))?;
            let schedule_json = serde_json::to_string(&orch.schedule)
                .map_err(|e| repo_error!("序列化调度配置失败: {}", e))?;

            conn.execute(
                "INSERT INTO orchestrations \
                 (id, name, description, created_at, updated_at, order_index, steps_json, schedule_json) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    orch.id,
                    orch.name,
                    orch.description,
                    orch.created_at,
                    orch.updated_at,
                    i,
                    steps_json,
                    schedule_json
                ],
            )
            .map_err(|e| repo_error!("插入编排失败: {}", e))?;
        }

        Ok(())
    }

    fn write_orchestration_runs(
        conn: &Connection,
        runs: &[OrchestrationRun],
    ) -> Result<(), String> {
        conn.execute("DELETE FROM orchestration_runs", [])
            .map_err(|e| repo_error!("清除编排执行记录失败: {}", e))?;

        for run in runs {
            let steps_json = serde_json::to_string(&run.steps)
                .map_err(|e| repo_error!("序列化步骤结果失败: {}", e))?;

            conn.execute(
                "INSERT INTO orchestration_runs \
                 (id, orchestration_id, status, start_time, end_time, \
                  total_time, success_count, failed_count, skipped_count, steps_json) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    run.id,
                    run.orchestration_id,
                    run.status,
                    run.start_time,
                    run.end_time,
                    run.total_time as i64,
                    run.success_count as i64,
                    run.failed_count as i64,
                    run.skipped_count as i64,
                    steps_json
                ],
            )
            .map_err(|e| repo_error!("插入编排执行记录失败: {}", e))?;
        }

        Ok(())
    }

    fn write_stress_configs(
        conn: &Connection,
        configs: &[StressConfigExport],
    ) -> Result<(), String> {
        conn.execute("DELETE FROM stress_configs", [])
            .map_err(|e| repo_error!("清除压测配置失败: {}", e))?;

        for config in configs {
            let assertions_json = serde_json::to_string(&config.assertions)
                .map_err(|e| repo_error!("序列化断言失败: {}", e))?;

            conn.execute(
                "INSERT INTO stress_configs \
                 (api_id, concurrent, total_requests, duration_seconds, ramp_up_seconds, timeout_ms, assertions_json) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    config.api_id,
                    config.concurrent as i32,
                    config.total_requests.map(|v| v as i64),
                    config.duration_seconds.map(|v| v as i32),
                    config.ramp_up_seconds as i32,
                    config.timeout_ms as i64,
                    assertions_json,
                ],
            )
            .map_err(|e| repo_error!("插入压测配置失败: {}", e))?;
        }

        Ok(())
    }

    fn write_stress_results(conn: &Connection, results: &[StressTestResult]) -> Result<(), String> {
        conn.execute("DELETE FROM stress_results", [])
            .map_err(|e| repo_error!("清除压测结果失败: {}", e))?;

        for result in results {
            let config_json = serde_json::to_string(&result.config)
                .map_err(|e| repo_error!("序列化压测配置失败: {}", e))?;
            let status_distribution_json = serde_json::to_string(&result.status_distribution)
                .map_err(|e| repo_error!("序列化状态分布失败: {}", e))?;
            let error_distribution_json = serde_json::to_string(&result.error_distribution)
                .map_err(|e| repo_error!("序列化错误分布失败: {}", e))?;

            conn.execute(
                "INSERT INTO stress_results \
                 (id, api_id, config_json, start_time, end_time, \
                  total_requests, successful_requests, failed_requests, \
                  total_time_ms, qps, avg_time_ms, min_time_ms, max_time_ms, \
                  p50_time_ms, p90_time_ms, p95_time_ms, p99_time_ms, \
                  success_rate, status_distribution_json, error_distribution_json) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
                params![
                    result.id,
                    result.api_id,
                    config_json,
                    result.start_time,
                    result.end_time,
                    result.total_requests as i64,
                    result.successful_requests as i64,
                    result.failed_requests as i64,
                    result.total_time_ms as i64,
                    result.qps,
                    result.avg_time_ms,
                    result.min_time_ms as i64,
                    result.max_time_ms as i64,
                    result.p50_time_ms,
                    result.p90_time_ms,
                    result.p95_time_ms,
                    result.p99_time_ms,
                    result.success_rate,
                    status_distribution_json,
                    error_distribution_json,
                ],
            )
            .map_err(|e| repo_error!("插入压测结果失败: {}", e))?;

            for detail in &result.failed_request_details {
                let detail_id = crate::domain::models::generate_id("stress_detail");
                conn.execute(
                    "INSERT INTO stress_result_details \
                     (id, result_id, time, error, status, elapsed_ms) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        detail_id,
                        result.id,
                        detail.time,
                        detail.error,
                        detail.status.map(|v| v as i32),
                        detail.elapsed_ms as i64,
                    ],
                )
                .map_err(|e| repo_error!("插入压测失败详情失败: {}", e))?;
            }

            for point in &result.history {
                let point_id = crate::domain::models::generate_id("stress_point");
                conn.execute(
                    "INSERT INTO stress_result_points \
                     (id, result_id, second, qps, avg_time_ms, successful, failed, requests, concurrent) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        point_id,
                        result.id,
                        point.second as i32,
                        point.qps,
                        point.avg_time_ms,
                        point.successful as i64,
                        point.failed as i64,
                        point.requests as i64,
                        point.concurrent as i32,
                    ],
                )
                .map_err(|e| repo_error!("插入压测历史数据点失败: {}", e))?;
            }
        }

        Ok(())
    }

    fn write_docs(conn: &Connection, docs: &[DocExportItem]) -> Result<(), String> {
        conn.execute("DELETE FROM docs", [])
            .map_err(|e| repo_error!("清除文档失败: {}", e))?;

        for doc in docs {
            conn.execute(
                "INSERT INTO docs (api_id, updated_at, content) VALUES (?1, ?2, ?3)",
                params![doc.api_id, doc.updated_at, doc.content],
            )
            .map_err(|e| repo_error!("插入文档失败: {}", e))?;
        }

        Ok(())
    }

    fn write_chat_sessions(conn: &Connection, sessions: &[ChatSession]) -> Result<(), String> {
        conn.execute("DELETE FROM chat_sessions", [])
            .map_err(|e| repo_error!("清除聊天会话失败: {}", e))?;

        for session in sessions {
            conn.execute(
                "INSERT INTO chat_sessions (id, created_at, title, active_session) \
                 VALUES (?1, ?2, ?3, 0)",
                params![session.id, session.created_at, session.title],
            )
            .map_err(|e| repo_error!("插入聊天会话失败: {}", e))?;

            for msg in &session.messages {
                let msg_id = crate::domain::models::generate_id("chat_msg");
                conn.execute(
                    "INSERT INTO chat_messages (id, session_id, role, content, reasoning, timestamp) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        msg_id,
                        session.id,
                        msg.role,
                        msg.content,
                        msg.reasoning,
                        msg.timestamp,
                    ],
                )
                .map_err(|e| repo_error!("插入聊天消息失败: {}", e))?;
            }
        }

        Ok(())
    }

    fn write_ws_configs(conn: &Connection, configs: &[WsConfigItem]) -> Result<(), String> {
        conn.execute("DELETE FROM ws_configs", [])
            .map_err(|e| repo_error!("清除 WebSocket 配置失败: {}", e))?;

        for (i, config) in configs.iter().enumerate() {
            let headers_json = serde_json::to_string(&config.headers)
                .map_err(|e| repo_error!("序列化 WebSocket 请求头失败: {}", e))?;
            let params_json = serde_json::to_string(&config.params)
                .map_err(|e| repo_error!("序列化 WebSocket 参数失败: {}", e))?;

            conn.execute(
                "INSERT INTO ws_configs \
                 (id, name, url, headers_json, params_json, created_at, updated_at, order_index) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    config.id,
                    config.name,
                    config.url,
                    headers_json,
                    params_json,
                    config.created_at,
                    config.updated_at,
                    i,
                ],
            )
            .map_err(|e| repo_error!("插入 WebSocket 配置失败: {}", e))?;
        }

        Ok(())
    }

    fn write_app_state(
        conn: &Connection,
        state: &AppStateExport,
        active_environment_id: &Option<String>,
    ) -> Result<(), String> {
        conn.execute("DELETE FROM app_state", [])
            .map_err(|e| repo_error!("清除应用状态失败: {}", e))?;

        let expanded_ids_json = serde_json::to_string(&state.expanded_ids)
            .map_err(|e| repo_error!("序列化展开ID失败: {}", e))?;

        let open_tabs_data: Vec<serde_json::Value> = state
            .open_tabs
            .iter()
            .map(|tab| serde_json::json!({"id": tab.id, "type": tab.tab_type}))
            .collect();
        let open_tabs_json = serde_json::to_string(&open_tabs_data)
            .map_err(|e| repo_error!("序列化标签页失败: {}", e))?;

        let request_tabs_json = serde_json::to_string(&state.request_tabs)
            .map_err(|e| repo_error!("序列化请求标签页失败: {}", e))?;

        conn.execute(
            "INSERT INTO app_state \
             (id, expanded_ids_json, open_tabs_json, active_tab_index, request_tabs_json, active_environment_id) \
             VALUES (1, ?1, ?2, ?3, ?4, ?5)",
            params![
                expanded_ids_json,
                open_tabs_json,
                state.active_tab_index as i32,
                request_tabs_json,
                active_environment_id,
            ],
        )
        .map_err(|e| repo_error!("写入应用状态失败: {}", e))?;

        Ok(())
    }
}
