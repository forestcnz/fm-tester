//! SQLite 编排仓储实现（精简版）
//!
//! 使用 JSON 列存储步骤和调度数据。

use crate::domain::models::{
    Orchestration, OrchestrationIndex, OrchestrationIndexEntry, OrchestrationRun,
    OrchestrationRunIndex, OrchestrationRunIndexEntry, OrchestrationSchedule, OrchestrationStep,
    StepRunResult,
};
use crate::domain::repositories::OrchestrationRepository;
use crate::infrastructure::data_dir;
use crate::infrastructure::sqlite::connection::{with_connection, with_transaction};
use crate::repo_error;
use rusqlite::params;
use std::path::PathBuf;

/// SQLite 编排仓储
pub struct SqliteOrchestrationRepository;

impl SqliteOrchestrationRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SqliteOrchestrationRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl OrchestrationRepository for SqliteOrchestrationRepository {
    fn read_index(&self, workspace_id: &str) -> Result<OrchestrationIndex, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, description, created_at, updated_at, steps_json \
                     FROM orchestrations \
                     ORDER BY order_index",
                )
                .map_err(|e| repo_error!("准备查询编排索引失败: {}", e))?;

            let rows: Vec<(String, String, Option<String>, String, String, String)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                    ))
                })
                .map_err(|e| repo_error!("查询编排索引失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析编排索引行数据失败: {}", e))?;

            let entries: Vec<OrchestrationIndexEntry> = rows
                .into_iter()
                .map(
                    |(id, name, description, created_at, updated_at, steps_json)| {
                        let steps: Vec<OrchestrationStep> = serde_json::from_str(&steps_json)
                            .map_err(|e| repo_error!("反序列化编排步骤失败: {}", e))?;
                        Ok(OrchestrationIndexEntry {
                            id,
                            name,
                            description,
                            step_count: steps.len(),
                            created_at,
                            updated_at,
                        })
                    },
                )
                .collect::<Result<Vec<_>, String>>()?;

            Ok(OrchestrationIndex {
                orchestrations: entries,
            })
        })
    }

    fn write_index(&self, workspace_id: &str, index: &OrchestrationIndex) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_transaction(&ws, |conn| {
            for (i, entry) in index.orchestrations.iter().enumerate() {
                conn.execute(
                    "UPDATE orchestrations SET name = ?1, description = ?2, order_index = ?3 WHERE id = ?4",
                    params![entry.name, entry.description, i as i32, entry.id],
                )
                .map_err(|e| repo_error!("更新编排索引条目失败: {}", e))?;
            }

            Ok(())
        })
    }

    fn read_orchestration(&self, workspace_id: &str, id: &str) -> Result<Orchestration, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let result = conn.query_row(
                "SELECT id, name, description, created_at, updated_at, steps_json, schedule_json \
                 FROM orchestrations \
                 WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                },
            );

            match result {
                Ok((id, name, description, created_at, updated_at, steps_json, schedule_json)) => {
                    let steps: Vec<OrchestrationStep> = serde_json::from_str(&steps_json)
                        .map_err(|e| repo_error!("反序列化编排步骤失败: {}", e))?;

                    let schedule: Option<OrchestrationSchedule> = if schedule_json == "{}" {
                        None
                    } else {
                        serde_json::from_str(&schedule_json)
                            .map_err(|e| repo_error!("反序列化编排调度配置失败: {}", e))?
                    };

                    Ok(Orchestration {
                        id,
                        name,
                        description,
                        created_at,
                        updated_at,
                        steps,
                        schedule,
                    })
                }
                Err(e) => Err(repo_error!("查询编排失败: {}", e)),
            }
        })
    }

    fn write_orchestration(
        &self,
        workspace_id: &str,
        orchestration: &Orchestration,
    ) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let steps_json = serde_json::to_string(&orchestration.steps)
                .map_err(|e| repo_error!("序列化步骤失败: {}", e))?;

            let schedule_json = serde_json::to_string(&orchestration.schedule)
                .map_err(|e| repo_error!("序列化调度配置失败: {}", e))?;

            conn.execute(
                "INSERT INTO orchestrations \
                 (id, name, description, created_at, updated_at, order_index, steps_json, schedule_json) \
                 VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7) \
                 ON CONFLICT(id) DO UPDATE SET \
                   name = excluded.name, \
                   description = excluded.description, \
                   created_at = excluded.created_at, \
                   updated_at = excluded.updated_at, \
                   steps_json = excluded.steps_json, \
                   schedule_json = excluded.schedule_json",
                params![
                    orchestration.id,
                    orchestration.name,
                    orchestration.description,
                    orchestration.created_at,
                    orchestration.updated_at,
                    steps_json,
                    schedule_json,
                ],
            )
            .map_err(|e| repo_error!("写入编排失败: {}", e))?;

            Ok(())
        })
    }

    fn delete_orchestration(&self, workspace_id: &str, id: &str) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute("DELETE FROM orchestrations WHERE id = ?1", params![id])
                .map_err(|e| repo_error!("删除编排失败: {}", e))?;
            Ok(())
        })
    }

    fn read_runs_index(
        &self,
        workspace_id: &str,
        orchestration_id: &str,
    ) -> Result<OrchestrationRunIndex, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, orchestration_id, status, start_time, total_time, \
                     success_count, failed_count, skipped_count \
                     FROM orchestration_runs \
                     WHERE orchestration_id = ?1 \
                     ORDER BY start_time DESC",
                )
                .map_err(|e| repo_error!("准备查询执行索引失败: {}", e))?;

            let entries: Vec<OrchestrationRunIndexEntry> = stmt
                .query_map(params![orchestration_id], |row| {
                    Ok(OrchestrationRunIndexEntry {
                        id: row.get(0)?,
                        orchestration_id: row.get(1)?,
                        status: row.get(2)?,
                        start_time: row.get(3)?,
                        total_time: row.get::<_, i32>(4)? as u64,
                        success_count: row.get::<_, i32>(5)? as usize,
                        failed_count: row.get::<_, i32>(6)? as usize,
                        skipped_count: row.get::<_, i32>(7)? as usize,
                    })
                })
                .map_err(|e| repo_error!("查询执行索引失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析执行索引行数据失败: {}", e))?;

            Ok(OrchestrationRunIndex { runs: entries })
        })
    }

    fn write_runs_index(
        &self,
        workspace_id: &str,
        _orchestration_id: &str,
        index: &OrchestrationRunIndex,
    ) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            for entry in &index.runs {
                conn.execute(
                    "INSERT OR REPLACE INTO orchestration_runs \
                     (id, orchestration_id, status, start_time, end_time, total_time, \
                      success_count, failed_count, skipped_count, steps_json) \
                     VALUES (?1, ?2, ?3, ?4, \
                     COALESCE((SELECT end_time FROM orchestration_runs WHERE id = ?1), ''), \
                     ?5, ?6, ?7, ?8, \
                     COALESCE((SELECT steps_json FROM orchestration_runs WHERE id = ?1), '[]'))",
                    params![
                        entry.id,
                        entry.orchestration_id,
                        entry.status,
                        entry.start_time,
                        entry.total_time as i32,
                        entry.success_count as i32,
                        entry.failed_count as i32,
                        entry.skipped_count as i32,
                    ],
                )
                .map_err(|e| repo_error!("写入执行索引条目失败: {}", e))?;
            }

            Ok(())
        })
    }

    fn read_run(
        &self,
        workspace_id: &str,
        orchestration_id: &str,
        run_id: &str,
    ) -> Result<OrchestrationRun, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let result = conn.query_row(
                "SELECT id, orchestration_id, status, start_time, end_time, total_time, \
                 success_count, failed_count, skipped_count, steps_json \
                 FROM orchestration_runs \
                 WHERE id = ?1 AND orchestration_id = ?2",
                params![run_id, orchestration_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i32>(5)?,
                        row.get::<_, i32>(6)?,
                        row.get::<_, i32>(7)?,
                        row.get::<_, i32>(8)?,
                        row.get::<_, String>(9)?,
                    ))
                },
            );

            match result {
                Ok((
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
                )) => {
                    let steps: Vec<StepRunResult> = serde_json::from_str(&steps_json)
                        .map_err(|e| repo_error!("反序列化编排运行步骤失败: {}", e))?;

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
                }
                Err(e) => Err(repo_error!("查询执行记录失败: {}", e)),
            }
        })
    }

    fn write_run(
        &self,
        workspace_id: &str,
        _orchestration_id: &str,
        run: &OrchestrationRun,
    ) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let steps_json = serde_json::to_string(&run.steps)
                .map_err(|e| repo_error!("序列化步骤执行结果失败: {}", e))?;

            conn.execute(
                "INSERT OR REPLACE INTO orchestration_runs \
                 (id, orchestration_id, status, start_time, end_time, total_time, \
                  success_count, failed_count, skipped_count, steps_json) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    run.id,
                    run.orchestration_id,
                    run.status,
                    run.start_time,
                    run.end_time,
                    run.total_time as i32,
                    run.success_count as i32,
                    run.failed_count as i32,
                    run.skipped_count as i32,
                    steps_json,
                ],
            )
            .map_err(|e| repo_error!("写入执行记录失败: {}", e))?;

            Ok(())
        })
    }

    fn delete_run(
        &self,
        workspace_id: &str,
        orchestration_id: &str,
        run_id: &str,
    ) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute(
                "DELETE FROM orchestration_runs WHERE id = ?1 AND orchestration_id = ?2",
                params![run_id, orchestration_id],
            )
            .map_err(|e| repo_error!("删除执行记录失败: {}", e))?;
            Ok(())
        })
    }

    fn clear_all_runs(&self, workspace_id: &str, orchestration_id: &str) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute(
                "DELETE FROM orchestration_runs WHERE orchestration_id = ?1",
                params![orchestration_id],
            )
            .map_err(|e| repo_error!("清空执行记录失败: {}", e))?;
            Ok(())
        })
    }

    fn get_orchestrations_dir(&self, workspace_id: &str) -> PathBuf {
        data_dir::get_workspace_db_path(workspace_id)
    }

    fn get_index_path(&self, workspace_id: &str) -> PathBuf {
        data_dir::get_workspace_db_path(workspace_id)
    }

    fn get_orchestration_path(&self, workspace_id: &str, _id: &str) -> PathBuf {
        data_dir::get_workspace_db_path(workspace_id)
    }

    fn get_runs_dir(&self, workspace_id: &str, _orchestration_id: &str) -> PathBuf {
        data_dir::get_workspace_db_path(workspace_id)
    }

    fn get_runs_index_path(&self, workspace_id: &str, _orchestration_id: &str) -> PathBuf {
        data_dir::get_workspace_db_path(workspace_id)
    }

    fn get_run_path(&self, workspace_id: &str, _orchestration_id: &str, _run_id: &str) -> PathBuf {
        data_dir::get_workspace_db_path(workspace_id)
    }
}
