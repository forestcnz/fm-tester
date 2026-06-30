//! SQLite 连接管理器
//!
//! 连接池模式：每个工作区维护一个 r2d2 连接池。
//! - `PooledConnection` 是 RAII 守卫，持有期间连接不会被回收，杜绝裸指针在切工作区时
//!   被释放导致的 use-after-free；
//! - 闭包内若再次调用 `with_connection` / `with_transaction`（重入），将从池中获取另一条
//!   独立连接，避免单连接可重入锁死锁；
//! - 切换工作区不再关闭旧连接，旧工作区的进行中操作仍持有各自连接，并发安全。
//!
//! 数据库文件路径：`./data/<workspace_id>/data.db`

use crate::infrastructure::data_dir;
use r2d2::{CustomizeConnection, Pool};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Mutex;

use super::schema::SCHEMA_SQL;

/// 每个工作区连接池的最大连接数。
/// 取 8 足以支撑常规并发与重入嵌套，同时控制文件句柄占用。
const MAX_POOL_SIZE: u32 = 8;

/// 全局连接池表：workspace_id -> 连接池。
static POOLS: Mutex<Option<HashMap<String, Pool<SqliteConnectionManager>>>> = Mutex::new(None);

/// 每条新连接的初始化逻辑：设置 WAL、启用外键约束、确保 schema 已建。
#[derive(Debug)]
struct SqliteCustomizer;

impl CustomizeConnection<Connection, rusqlite::Error> for SqliteCustomizer {
    fn on_acquire(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        // SCHEMA_SQL 全部为 CREATE TABLE IF NOT EXISTS，幂等；
        // 在每条新连接上执行可保证库结构始终就绪。
        conn.execute_batch(SCHEMA_SQL)?;
        Ok(())
    }
}

/// 获取（必要时创建）指定工作区的连接池。
/// 池在创建后被缓存，切换工作区不会销毁已有池，从而避免打断进行中的操作。
fn ensure_pool(workspace_id: &str) -> Result<Pool<SqliteConnectionManager>, String> {
    let mut guard = POOLS.lock().map_err(|e| format!("数据库锁中毒: {}", e))?;
    let map = guard.get_or_insert_with(HashMap::new);
    if let Some(pool) = map.get(workspace_id) {
        return Ok(pool.clone());
    }

    let db_path = data_dir::get_workspace_db_path(workspace_id);
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建数据库目录失败: {}", e))?;
    }

    tracing::debug!("创建工作区连接池: {} ({})", workspace_id, db_path.display());

    let manager = SqliteConnectionManager::file(&db_path);
    let pool = Pool::builder()
        .max_size(MAX_POOL_SIZE)
        .connection_customizer(Box::new(SqliteCustomizer))
        .build(manager)
        .map_err(|e| format!("创建连接池失败: {}", e))?;

    map.insert(workspace_id.to_string(), pool.clone());
    Ok(pool)
}

/// 在连接中执行闭包。
///
/// 从工作区连接池获取一条 `PooledConnection`（RAII 守卫），守卫在闭包返回前始终有效，
/// 彻底消除了原先裸指针在切工作区时被释放的 use-after-free。
/// 闭包内允许再次调用 `with_connection` / `with_transaction`（重入），
/// 此时将从池中获取另一条独立连接，不会死锁。
pub fn with_connection<T>(
    workspace_id: &str,
    f: impl FnOnce(&Connection) -> Result<T, String>,
) -> Result<T, String> {
    let pool = ensure_pool(workspace_id)?;
    let conn = pool
        .get()
        .map_err(|e| format!("获取数据库连接失败: {}", e))?;
    f(&conn)
}

/// 在事务中执行闭包，失败时自动 ROLLBACK，成功时 COMMIT。
///
/// 语义与 `with_connection` 一致，区别仅在于把闭包包裹在一个事务内。
/// 闭包内允许重入调用其它 `with_connection` / `with_transaction`。
pub fn with_transaction<T>(
    workspace_id: &str,
    f: impl FnOnce(&Connection) -> Result<T, String>,
) -> Result<T, String> {
    let pool = ensure_pool(workspace_id)?;
    let conn = pool
        .get()
        .map_err(|e| format!("获取数据库连接失败: {}", e))?;

    // unchecked_transaction 只需 &Connection（不触发 &mut 借用检查），
    // 失败时 Drop 自动 ROLLBACK，仅在显式 commit 时才提交。
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("开启事务失败: {}", e))?;
    let result = f(&tx);
    match result {
        Ok(v) => {
            tx.commit().map_err(|e| format!("提交事务失败: {}", e))?;
            Ok(v)
        }
        Err(e) => {
            // tx Drop 时会自动 ROLLBACK，这里显式调用以尽早释放写锁
            let _ = tx.rollback();
            Err(e)
        }
    }
}

/// 重置工作区数据库：删除所有用户表并按 SCHEMA_SQL 重新建表。
///
/// 用于覆盖恢复前清空目标工作区数据。db 文件与连接池保持不变。
/// 注意：`PRAGMA foreign_keys` 必须在事务外设置，故此处使用 `with_connection` 而非事务。
pub fn reset_workspace_schema(workspace_id: &str) -> Result<(), String> {
    with_connection(workspace_id, |conn| {
        conn.execute_batch("PRAGMA foreign_keys=OFF")
            .map_err(|e| format!("关闭外键约束失败: {}", e))?;

        // 动态获取所有用户表（排除 sqlite 内部表）
        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            )
            .map_err(|e| format!("查询表清单失败: {}", e))?;
        let table_names: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("读取表名失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);

        // 逐表删除（DROP 比 DELETE 更彻底，重置自增与结构）
        for name in &table_names {
            conn.execute(&format!("DROP TABLE IF EXISTS \"{}\"", name), [])
                .map_err(|e| format!("删除表 {} 失败: {}", name, e))?;
        }

        // 重建 schema
        conn.execute_batch(SCHEMA_SQL)
            .map_err(|e| format!("重建 schema 失败: {}", e))?;

        conn.execute_batch("PRAGMA foreign_keys=ON")
            .map_err(|e| format!("开启外键约束失败: {}", e))?;
        Ok(())
    })
}

/// 应用退出时调用：尽力对每个工作区做一次 WAL checkpoint，随后销毁所有连接池。
pub fn shutdown() {
    if let Ok(mut guard) = POOLS.lock() {
        if let Some(map) = guard.take() {
            tracing::info!("关闭 {} 个工作区连接池", map.len());
            for (ws_id, pool) in map {
                if let Ok(conn) = pool.get() {
                    let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
                }
                tracing::debug!("工作区连接池已关闭: {}", ws_id);
            }
        }
    }
}
