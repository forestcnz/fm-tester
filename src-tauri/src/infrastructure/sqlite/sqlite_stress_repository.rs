//! SQLite 压力测试仓储实现
//!
//! failed_request_details 和 history 拆分到独立表，支持分页查询。

use crate::domain::models::{
    FailedRequest, HistoryPoint, StressParamsConfig, StressTestConfig, StressTestResult,
    StressTestResultIndexEntry,
};
use crate::domain::repositories::StressTestRepository;
use crate::infrastructure::sqlite::connection::{with_connection, with_transaction};
use crate::repo_error;
use std::collections::HashMap;

pub struct SqliteStressRepository;

impl SqliteStressRepository {
    pub fn new() -> Self {
        Self
    }

    fn ensure_config_row(workspace_id: &str, api_id: &str) -> Result<(), String> {
        let ws = workspace_id.to_string();
        let aid = api_id.to_string();
        with_connection(&ws, |conn| {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM stress_configs WHERE api_id = ?1",
                    rusqlite::params![&aid],
                    |row| row.get::<_, bool>(0),
                )
                .map_err(|e| repo_error!("查询压测配置行失败: {}", e))?;

            if !exists {
                let default = StressParamsConfig::default();
                conn.execute(
                    "INSERT INTO stress_configs (api_id, concurrent, total_requests, duration_seconds, ramp_up_seconds, timeout_ms) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        &aid,
                        default.concurrent as i32,
                        default.total_requests.map(|v| v as i64),
                        default.duration_seconds.map(|v| v as i32),
                        default.ramp_up_seconds as i32,
                        default.timeout_ms as i64,
                    ],
                )
                .map_err(|e| repo_error!("插入默认压测配置失败: {}", e))?;
            }
            Ok(())
        })
    }

    fn parse_distribution_json(json: &str) -> Result<HashMap<String, u64>, String> {
        if json.is_empty() || json == "{}" {
            return Ok(HashMap::new());
        }
        serde_json::from_str::<HashMap<String, u64>>(json)
            .map_err(|e| repo_error!("反序列化压测状态分布失败: {}", e))
    }

    fn read_failed_details(
        workspace_id: &str,
        result_id: &str,
    ) -> Result<Vec<FailedRequest>, String> {
        let ws = workspace_id.to_string();
        let rid = result_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT time, error, status, elapsed_ms \
                     FROM stress_result_details WHERE result_id = ?1 ORDER BY rowid",
                )
                .map_err(|e| repo_error!("准备查询压测失败详情失败: {}", e))?;

            let rows = stmt
                .query_map(rusqlite::params![&rid], |row| {
                    Ok(FailedRequest {
                        time: row.get::<_, String>(0)?,
                        error: row.get::<_, String>(1)?,
                        status: row.get::<_, Option<i32>>(2)?.map(|v| v as u16),
                        elapsed_ms: row.get::<_, i32>(3)? as u64,
                    })
                })
                .map_err(|e| repo_error!("查询压测失败详情失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析压测失败详情行数据失败: {}", e))?;

            Ok(rows)
        })
    }

    fn read_history_points(
        workspace_id: &str,
        result_id: &str,
    ) -> Result<Vec<HistoryPoint>, String> {
        let ws = workspace_id.to_string();
        let rid = result_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT second, qps, avg_time_ms, successful, failed, requests, concurrent \
                     FROM stress_result_points WHERE result_id = ?1 ORDER BY second",
                )
                .map_err(|e| repo_error!("准备查询压测历史数据点失败: {}", e))?;

            let rows = stmt
                .query_map(rusqlite::params![&rid], |row| {
                    Ok(HistoryPoint {
                        second: row.get::<_, i32>(0)? as u32,
                        qps: row.get::<_, f64>(1)?,
                        avg_time_ms: row.get::<_, f64>(2)?,
                        successful: row.get::<_, i32>(3)? as u64,
                        failed: row.get::<_, i32>(4)? as u64,
                        requests: row.get::<_, i32>(5)? as u64,
                        concurrent: row.get::<_, i32>(6)? as u32,
                    })
                })
                .map_err(|e| repo_error!("查询压测历史数据点失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析压测历史数据点行数据失败: {}", e))?;

            Ok(rows)
        })
    }
}

impl Default for SqliteStressRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl StressTestRepository for SqliteStressRepository {
    fn read_params(&self, workspace_id: &str, api_id: &str) -> Result<StressParamsConfig, String> {
        let ws = workspace_id.to_string();
        let aid = api_id.to_string();
        with_connection(&ws, |conn| {
            let result = conn.query_row(
                "SELECT concurrent, total_requests, duration_seconds, ramp_up_seconds, timeout_ms \
                 FROM stress_configs WHERE api_id = ?1",
                rusqlite::params![&aid],
                |row| {
                    Ok(StressParamsConfig {
                        concurrent: row.get::<_, i32>(0)? as u32,
                        total_requests: row.get::<_, Option<i64>>(1)?.map(|v| v as u64),
                        duration_seconds: row.get::<_, Option<i32>>(2)?.map(|v| v as u32),
                        ramp_up_seconds: row.get::<_, i32>(3)? as u32,
                        timeout_ms: row.get::<_, i64>(4)? as u64,
                    })
                },
            );

            match result {
                Ok(config) => Ok(config),
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    let _ = conn;
                    Self::ensure_config_row(workspace_id, api_id)?;
                    Ok(StressParamsConfig::default())
                }
                Err(e) => Err(repo_error!("读取压测参数失败: {}", e)),
            }
        })
    }

    fn save_params(
        &self,
        workspace_id: &str,
        api_id: &str,
        config: &StressParamsConfig,
    ) -> Result<(), String> {
        let ws = workspace_id.to_string();
        let aid = api_id.to_string();
        with_connection(&ws, |conn| {
            Self::ensure_config_row(workspace_id, &aid)?;

            conn.execute(
                "UPDATE stress_configs SET \
                 concurrent = ?1, total_requests = ?2, duration_seconds = ?3, \
                 ramp_up_seconds = ?4, timeout_ms = ?5 \
                 WHERE api_id = ?6",
                rusqlite::params![
                    config.concurrent as i32,
                    config.total_requests.map(|v| v as i64),
                    config.duration_seconds.map(|v| v as i32),
                    config.ramp_up_seconds as i32,
                    config.timeout_ms as i64,
                    &aid,
                ],
            )
            .map_err(|e| repo_error!("保存压测参数失败: {}", e))?;

            Ok(())
        })
    }

    fn save_result(&self, workspace_id: &str, result: &StressTestResult) -> Result<(), String> {
        let ws = workspace_id.to_string();
        let config_json = serde_json::to_string(&result.config)
            .map_err(|e| repo_error!("序列化压测配置失败: {}", e))?;
        let status_distribution_json = serde_json::to_string(&result.status_distribution)
            .map_err(|e| repo_error!("序列化状态分布失败: {}", e))?;
        let error_distribution_json = serde_json::to_string(&result.error_distribution)
            .map_err(|e| repo_error!("序列化错误分布失败: {}", e))?;

        let result_id = result.id.clone();
        let api_id = result.api_id.clone().unwrap_or_default();
        let start_time = result.start_time.clone();
        let end_time = result.end_time.clone();
        let total_requests = result.total_requests;
        let successful_requests = result.successful_requests;
        let failed_requests = result.failed_requests;
        let total_time_ms = result.total_time_ms;
        let qps = result.qps;
        let avg_time_ms = result.avg_time_ms;
        let min_time_ms = result.min_time_ms;
        let max_time_ms = result.max_time_ms;
        let p50 = result.p50_time_ms;
        let p90 = result.p90_time_ms;
        let p95 = result.p95_time_ms;
        let p99 = result.p99_time_ms;
        let success_rate = result.success_rate;

        let details = result.failed_request_details.clone();
        let points = result.history.clone();

        with_transaction(&ws, |conn| {
            conn.execute(
                "INSERT INTO stress_results \
                 (id, api_id, config_json, start_time, end_time, \
                  total_requests, successful_requests, failed_requests, \
                  total_time_ms, qps, avg_time_ms, min_time_ms, max_time_ms, \
                  p50_time_ms, p90_time_ms, p95_time_ms, p99_time_ms, \
                  success_rate, status_distribution_json, error_distribution_json) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
                rusqlite::params![
                    &result_id,
                    &api_id,
                    &config_json,
                    &start_time,
                    &end_time,
                    total_requests as i64,
                    successful_requests as i64,
                    failed_requests as i64,
                    total_time_ms as i64,
                    qps,
                    avg_time_ms,
                    min_time_ms as i64,
                    max_time_ms as i64,
                    p50,
                    p90,
                    p95,
                    p99,
                    success_rate,
                    &status_distribution_json,
                    &error_distribution_json,
                ],
            )
            .map_err(|e| repo_error!("插入压测结果失败: {}", e))?;

            for detail in &details {
                conn.execute(
                    "INSERT INTO stress_result_details \
                     (id, result_id, time, error, status, elapsed_ms) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        crate::domain::models::generate_id("stress_detail"),
                        &result_id,
                        &detail.time,
                        &detail.error,
                        detail.status.map(|v| v as i32),
                        detail.elapsed_ms as i64,
                    ],
                )
                .map_err(|e| repo_error!("插入压测失败详情失败: {}", e))?;
            }

            for point in &points {
                conn.execute(
                    "INSERT INTO stress_result_points \
                     (id, result_id, second, qps, avg_time_ms, successful, failed, requests, concurrent) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    rusqlite::params![
                        crate::domain::models::generate_id("stress_point"),
                        &result_id,
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

            Ok(())
        })
    }

    fn read_result(
        &self,
        workspace_id: &str,
        _api_id: &str,
        id: &str,
    ) -> Result<Option<StressTestResult>, String> {
        let ws = workspace_id.to_string();
        let rid = id.to_string();
        with_connection(&ws, |conn| {
            let result = conn.query_row(
                "SELECT id, api_id, config_json, start_time, end_time, \
                 total_requests, successful_requests, failed_requests, \
                 total_time_ms, qps, avg_time_ms, min_time_ms, max_time_ms, \
                 p50_time_ms, p90_time_ms, p95_time_ms, p99_time_ms, \
                 success_rate, status_distribution_json, error_distribution_json \
                 FROM stress_results WHERE id = ?1",
                rusqlite::params![&rid],
                |row| {
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
                },
            );

            match result {
                Ok((
                    result_id,
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
                )) => {
                    let config: StressTestConfig = serde_json::from_str(&config_json)
                        .map_err(|e| repo_error!("反序列化压测配置失败: {}", e))?;
                    let status_distribution =
                        Self::parse_distribution_json(&status_distribution_json)?;
                    let error_distribution =
                        Self::parse_distribution_json(&error_distribution_json)?;

                    let _ = conn;
                    let failed_request_details =
                        Self::read_failed_details(workspace_id, &result_id)?;
                    let history = Self::read_history_points(workspace_id, &result_id)?;

                    Ok(Some(StressTestResult {
                        id: result_id,
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
                    }))
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(repo_error!("读取压测结果失败: {}", e)),
            }
        })
    }

    fn delete_result(&self, workspace_id: &str, _api_id: &str, id: &str) -> Result<(), String> {
        let ws = workspace_id.to_string();
        let rid = id.to_string();
        with_connection(&ws, |conn| {
            conn.execute(
                "DELETE FROM stress_results WHERE id = ?1",
                rusqlite::params![&rid],
            )
            .map_err(|e| repo_error!("删除压测结果失败: {}", e))?;
            Ok(())
        })
    }

    fn get_api_results(
        &self,
        workspace_id: &str,
        api_id: &str,
    ) -> Result<Vec<StressTestResultIndexEntry>, String> {
        let ws = workspace_id.to_string();
        let aid = api_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, api_id, config_json, start_time, qps, success_rate, total_requests \
                     FROM stress_results WHERE api_id = ?1 ORDER BY start_time DESC",
                )
                .map_err(|e| repo_error!("准备查询压测结果列表失败: {}", e))?;

            let rows: Vec<(String, Option<String>, String, String, f64, f64, i64)> = stmt
                .query_map(rusqlite::params![&aid], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, f64>(4)?,
                        row.get::<_, f64>(5)?,
                        row.get::<_, i64>(6)?,
                    ))
                })
                .map_err(|e| repo_error!("查询压测结果列表失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析压测结果行数据失败: {}", e))?;

            let entries: Vec<StressTestResultIndexEntry> = rows
                .into_iter()
                .map(
                    |(id, api_id, config_json, start_time, qps, success_rate, total_requests)| {
                        let config: StressTestConfig = serde_json::from_str(&config_json)
                            .map_err(|e| repo_error!("反序列化压测配置失败: {}", e))?;
                        Ok(StressTestResultIndexEntry {
                            id,
                            api_id,
                            api_name: config.api_name,
                            start_time,
                            qps,
                            success_rate,
                            total_requests: total_requests as u64,
                        })
                    },
                )
                .collect::<Result<Vec<_>, String>>()?;

            Ok(entries)
        })
    }
}
