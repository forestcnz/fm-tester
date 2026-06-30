# FM Tester 后端存储重设计方案

> 版本: 4.0 | 日期: 2026-06-15 | 状态: 草案

---

## 一、现状分析

### 1.1 当前架构概览

```
{exe_dir}/data/
├── config.json                        ← 全局配置 (JSON 文件) ✅ 保留
│   ├── settings (超时/语言/AI 配置)
│   ├── workspaces[] (工作区列表)
│   └── last_workspace_id
│
├── .encryption_key                    ← 加密密钥文件
│
└── data_<workspace_id>.db             ← 每个工作区独立 SQLite
    ├── environments (含 JSON 列)
    ├── app_state (含 JSON 列)
    ├── ws_configs (含 JSON 列)
    ├── cookies
    ├── collection_items (含 JSON 列)
    ├── history_entries (含 JSON 列)
    ├── scripts
    ├── saved_responses
    ├── chat_sessions (含 JSON 列)
    ├── orchestrations (含 JSON 列)
    ├── orchestration_runs (含 JSON 列)
    ├── stress_configs (含 JSON 列)
    ├── stress_results (含 JSON 列)
    └── docs
```

### 1.2 核心痛点

| # | 问题 | 影响 |
|---|------|------|
| **1** | **JSON-in-Column 反模式** — headers、variables、params、steps 等子数据序列化为 JSON 文本存列中 | 无法对子数据建索引和 SQL 查询；数据完整性靠应用层保证；JSON 反序列化开销大 |
| **2** | **无连接管理** — 每次操作 `Connection::open_with_flags()` 获取新连接，用完丢弃 | 频繁文件打开/关闭；连接泄漏风险 |
| **3** | **数据目录耦合** — 数据存储在 exe 同级 `./data/` 目录 | Windows 下 Program Files 不可写；多用户冲突；不符合各平台规范 |
| **4** | **加密密钥明文存文件** — `.encryption_key` 文件无额外保护 | 安全审计风险 |

### 1.3 当前 Schema JSON 列统计

| 表 | 行数估计 | JSON 列数 | 主要 JSON 列 |
|----|---------|----------|-------------|
| environments | < 20 | 2 | variables_json, common_headers_json |
| app_state | 1 | 3 | expanded_ids_json, open_tabs_json, request_tabs_json |
| collection_items | < 2000 | 7 | params, headers, form_fields, form_files, common_headers, variables, saved_response_ids |
| history_entries | < 10000 | 3 | request_headers, response_headers, form_fields |
| orchestrations | < 100 | 2 | steps_json, schedule_json |
| orchestration_runs | < 1000 | 1 | steps_json |
| stress_configs | < 100 | 1 | assertions_json |
| stress_results | < 500 | 5 | config_json, status_distribution, error_distribution, failed_requests, history_points |
| chat_sessions | < 500 | 1 | messages_json |

---

## 二、新方案设计目标

1. **保留 config.json** — 全局配置继续用 JSON 文件，不合并到数据库
2. **保留工作区隔离** — 每个工作区一个独立 db 文件，简单可控
3. **保留 TEXT 内容存 DB** — 脚本内容、文档内容继续存数据库 TEXT 列，不用文件系统
4. **规范化 Schema** — 消除 JSON-in-Column，子数据用关系表，保证数据完整性和可查询性
5. **单例长连接** — 每个工作区保持一个长连接，切换工作区时关闭旧连接、打开新连接
6. **跨平台数据目录** — 数据目录与旧版保持一致

---

## 三、新架构设计

### 3.1 总体架构

```
{exe_dir}/data/                           ← 应用同级 data 目录
│
├── config.json                           ← 全局配置（保留不变）
│   ├── settings
│   ├── workspaces[]
│   └── last_workspace_id
│
├── .encryption_key                       ← 加密密钥
│
└── <workspace_id>/                       ← 工作区子目录
    └── data.db                           ← 工作区 SQLite
```

### 3.2 与旧版路径对比

| 内容 | 旧路径 | 新路径 |
|------|--------|--------|
| 全局配置 | `./data/config.json` | `./data/config.json` |
| 加密密钥 | `./data/.encryption_key` | `./data/.encryption_key` |
| 工作区 DB | `./data/data_<id>.db` | `./data/<id>/data.db` |

### 3.4 存储策略矩阵

| 数据类型 | 存储位置 | 原因 |
|---------|---------|------|
| 全局配置 | `config.json` | 轻量、启动即需、人工可读可改 |
| 结构化数据 (关系) | SQLite 表 | 需要索引、查询、外键约束 |
| 文本内容 (脚本/文档/响应Body) | SQLite TEXT 列 | 数据与元数据一体，事务一致 |
| 加密数据 (API Key) | config.json (AES-256-GCM) | 在全局配置中，与 AI 设置同文件 |
| UI 运行时状态 | SQLite (3 个 JSON 列) | 结构不稳定、无需查询 |

---

## 四、完整数据库 Schema

> 以下为一个工作区 `data.db` 内的全部表。每个 db 只对应一个工作区，不需要 `workspace_id` 外键。
>
> **设计原则**: 只拆分**真正需要独立查询/分页**的数据。headers、params、variables、form_fields、steps 等是前端整体读写的列表，保留 JSON 列是最佳选择。

### 4.1 环境

```sql
CREATE TABLE environments (
    id                 TEXT PRIMARY KEY,
    name               TEXT NOT NULL UNIQUE,
    variables_json     TEXT NOT NULL DEFAULT '[]',    -- [{key, value, enabled, description}]
    common_headers_json TEXT NOT NULL DEFAULT '[]',   -- [{key, value, enabled, description}]
    order_index        INTEGER NOT NULL DEFAULT 0
);
```

### 4.2 集合/API (核心树结构)

```sql
CREATE TABLE collections (
    id                       TEXT PRIMARY KEY,
    parent_id                TEXT REFERENCES collections(id) ON DELETE CASCADE,
    name                     TEXT NOT NULL,
    description              TEXT NOT NULL DEFAULT '',
    item_type                TEXT NOT NULL CHECK (item_type IN ('collection', 'api')),
    -- API 专属
    method                   TEXT,
    url                      TEXT,
    body                     TEXT,
    body_type                TEXT CHECK (body_type IN ('none', 'form-data', 'x-www-form-urlencoded', 'raw', 'binary')),
    -- JSON 列表
    params_json              TEXT NOT NULL DEFAULT '[]',    -- [{key, value, enabled, description}]
    headers_json             TEXT NOT NULL DEFAULT '[]',    -- [{key, value, enabled, description}]
    form_fields_json         TEXT NOT NULL DEFAULT '[]',    -- [{key, value, field_type, enabled, description}]
    form_files_json          TEXT NOT NULL DEFAULT '[]',    -- [{field_id, file_path, file_name}]
    common_headers_json      TEXT NOT NULL DEFAULT '[]',    -- [{key, value, enabled, description}]
    variables_json           TEXT NOT NULL DEFAULT '[]',    -- [{key, value, enabled, description}]
    saved_response_ids_json  TEXT NOT NULL DEFAULT '[]',    -- [id, ...]
    order_index              INTEGER NOT NULL DEFAULT 0,
    created_at               TEXT NOT NULL,
    updated_at               TEXT NOT NULL
);

CREATE INDEX idx_col_parent ON collections(parent_id);
CREATE INDEX idx_col_type   ON collections(item_type);
```

### 4.3 WebSocket 配置

```sql
CREATE TABLE ws_configs (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    url          TEXT NOT NULL,
    headers_json TEXT NOT NULL DEFAULT '[]',   -- [{key, value, enabled}]
    params_json  TEXT NOT NULL DEFAULT '[]',   -- [{key, value, enabled}]
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    order_index  INTEGER NOT NULL DEFAULT 0
);
```

### 4.4 历史记录

```sql
CREATE TABLE history_entries (
    id                     TEXT PRIMARY KEY,
    method                 TEXT NOT NULL,
    url                    TEXT NOT NULL,
    resolved_url           TEXT NOT NULL DEFAULT '',
    status                 INTEGER NOT NULL,
    status_text            TEXT NOT NULL DEFAULT '',
    response_body          TEXT NOT NULL DEFAULT '',
    time_ms                INTEGER NOT NULL DEFAULT 0,
    size_bytes             INTEGER NOT NULL DEFAULT 0,
    created_at             TEXT NOT NULL,
    body                   TEXT,
    body_type              TEXT,
    api_id                 TEXT REFERENCES collections(id) ON DELETE SET NULL,
    api_name               TEXT NOT NULL DEFAULT '',
    date                   TEXT NOT NULL,
    request_headers_json   TEXT NOT NULL DEFAULT '[]',   -- [{key, value, enabled}]
    response_headers_json  TEXT NOT NULL DEFAULT '[]',   -- [{key, value}]
    form_fields_json       TEXT NOT NULL DEFAULT '[]'    -- [{key, value, field_type}]
);

CREATE INDEX idx_history_date ON history_entries(date);
CREATE INDEX idx_history_api  ON history_entries(api_id);
CREATE INDEX idx_history_time ON history_entries(created_at);
```

### 4.5 Cookie

```sql
CREATE TABLE cookies (
    name       TEXT NOT NULL,
    domain     TEXT NOT NULL,
    path       TEXT NOT NULL DEFAULT '/',
    value      TEXT NOT NULL DEFAULT '',
    expires    TEXT,
    max_age    INTEGER,
    secure     INTEGER NOT NULL DEFAULT 0,
    http_only  INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    PRIMARY KEY (name, domain)
);
```

### 4.6 脚本

```sql
CREATE TABLE scripts (
    id          TEXT PRIMARY KEY,
    target_type TEXT NOT NULL CHECK (target_type IN ('api', 'collection', 'workspace', 'environment')),
    target_id   TEXT,
    script_kind TEXT NOT NULL CHECK (script_kind IN ('pre', 'post')),
    filename    TEXT NOT NULL,
    content     TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    UNIQUE(target_type, target_id, script_kind)
);

CREATE INDEX idx_script_target ON scripts(target_type, target_id);
```

### 4.7 保存的响应

```sql
CREATE TABLE saved_responses (
    id          TEXT PRIMARY KEY,
    api_id      TEXT REFERENCES collections(id) ON DELETE SET NULL,
    name        TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    doc_content TEXT NOT NULL DEFAULT ''
);

CREATE INDEX idx_saved_resp_api ON saved_responses(api_id);
```

### 4.8 聊天（拆分 messages — 消息量大，需分页查询）

```sql
CREATE TABLE chat_sessions (
    id         TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    title      TEXT NOT NULL DEFAULT '',
    active     INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE chat_messages (
    id         TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    role       TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content    TEXT NOT NULL DEFAULT '',
    reasoning  TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);
CREATE INDEX idx_chat_msg_session ON chat_messages(session_id);
```

### 4.9 编排

```sql
CREATE TABLE orchestrations (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    description  TEXT NOT NULL DEFAULT '',
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    order_index  INTEGER NOT NULL DEFAULT 0,
    steps_json   TEXT NOT NULL DEFAULT '[]',     -- [{id, api_id, order_index, name, variable_key, capture_from, conditions}]
    schedule_json TEXT NOT NULL DEFAULT '{}'     -- {enabled, cron_expression, last_run_at, next_run_at}
);

CREATE TABLE orchestration_runs (
    id               TEXT PRIMARY KEY,
    orchestration_id TEXT NOT NULL REFERENCES orchestrations(id) ON DELETE CASCADE,
    status           TEXT NOT NULL CHECK (status IN ('running', 'success', 'failed', 'cancelled')),
    trigger_type     TEXT NOT NULL DEFAULT 'manual' CHECK (trigger_type IN ('manual', 'schedule')),
    start_time       TEXT NOT NULL,
    end_time         TEXT,
    total_time_ms    INTEGER NOT NULL DEFAULT 0,
    success_count    INTEGER NOT NULL DEFAULT 0,
    failed_count     INTEGER NOT NULL DEFAULT 0,
    skipped_count    INTEGER NOT NULL DEFAULT 0,
    steps_json       TEXT NOT NULL DEFAULT '[]'   -- [{step_id, order_index, step_name, status, request_method, request_url, ...}]
);
CREATE INDEX idx_orch_run_orch ON orchestration_runs(orchestration_id);
```

### 4.10 压力测试（拆分 details 和 points — 数据量大，需分页查询）

```sql
CREATE TABLE stress_configs (
    api_id           TEXT PRIMARY KEY REFERENCES collections(id) ON DELETE CASCADE,
    concurrent       INTEGER NOT NULL DEFAULT 10,
    total_requests   INTEGER,
    duration_seconds INTEGER,
    ramp_up_seconds  INTEGER NOT NULL DEFAULT 0,
    timeout_ms       INTEGER NOT NULL DEFAULT 30000,
    assertions_json  TEXT NOT NULL DEFAULT '[]'   -- [{field, operator, value}]
);

CREATE TABLE stress_results (
    id                  TEXT PRIMARY KEY,
    api_id              TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    config_json         TEXT NOT NULL,
    start_time          TEXT NOT NULL,
    end_time            TEXT,
    total_requests      INTEGER NOT NULL DEFAULT 0,
    successful_requests INTEGER NOT NULL DEFAULT 0,
    failed_requests     INTEGER NOT NULL DEFAULT 0,
    total_time_ms       INTEGER NOT NULL DEFAULT 0,
    qps                 REAL    NOT NULL DEFAULT 0.0,
    avg_time_ms         REAL    NOT NULL DEFAULT 0.0,
    min_time_ms         INTEGER NOT NULL DEFAULT 0,
    max_time_ms         INTEGER NOT NULL DEFAULT 0,
    p50_time_ms         REAL    NOT NULL DEFAULT 0.0,
    p90_time_ms         REAL    NOT NULL DEFAULT 0.0,
    p95_time_ms         REAL    NOT NULL DEFAULT 0.0,
    p99_time_ms         REAL    NOT NULL DEFAULT 0.0,
    success_rate        REAL    NOT NULL DEFAULT 0.0,
    status_distribution TEXT    NOT NULL DEFAULT '{}',
    error_distribution  TEXT    NOT NULL DEFAULT '{}'
);
CREATE INDEX idx_stress_res_api  ON stress_results(api_id);
CREATE INDEX idx_stress_res_time ON stress_results(start_time);

-- 拆分：失败请求详情（可能几千条，需分页）
CREATE TABLE stress_result_details (
    id               TEXT PRIMARY KEY,
    result_id        TEXT NOT NULL REFERENCES stress_results(id) ON DELETE CASCADE,
    request_index    INTEGER NOT NULL,
    request_method   TEXT NOT NULL,
    request_url      TEXT NOT NULL,
    response_status  INTEGER,
    response_body    TEXT,
    response_time_ms INTEGER,
    error_message    TEXT,
    success          INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX idx_stress_detail_res ON stress_result_details(result_id);

-- 拆分：时序数据点（可能几百个，需分页）
CREATE TABLE stress_result_points (
    id           TEXT PRIMARY KEY,
    result_id    TEXT NOT NULL REFERENCES stress_results(id) ON DELETE CASCADE,
    timestamp_ms INTEGER NOT NULL,
    qps          REAL NOT NULL DEFAULT 0.0,
    avg_time_ms  REAL NOT NULL DEFAULT 0.0,
    active_count INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_stress_point_res ON stress_result_points(result_id);
```

### 4.11 API 文档

```sql
CREATE TABLE docs (
    api_id     TEXT PRIMARY KEY REFERENCES collections(id) ON DELETE CASCADE,
    updated_at TEXT NOT NULL,
    content    TEXT NOT NULL DEFAULT ''
);
```

### 4.12 UI 状态记忆

```sql
CREATE TABLE app_state (
    id                   INTEGER PRIMARY KEY CHECK (id = 1),
    active_environment_id TEXT REFERENCES environments(id) ON DELETE SET NULL,
    expanded_ids_json    TEXT    NOT NULL DEFAULT '[]',
    open_tabs_json       TEXT    NOT NULL DEFAULT '[]',
    active_tab_index     INTEGER NOT NULL DEFAULT 0,
    request_tabs_json    TEXT    NOT NULL DEFAULT '{}'
);
```

### 4.13 Schema 总览

| 表名 | 说明 | JSON 列 | 变化 |
|------|------|---------|------|
| environments | 环境 | 2 (variables, common_headers) | 不变 |
| collections | 集合/API | 7 (params, headers, form_fields, form_files, common_headers, variables, saved_response_ids) | 不变 |
| ws_configs | WebSocket | 2 (headers, params) | 不变 |
| history_entries | 历史记录 | 3 (request_headers, response_headers, form_fields) | 不变 |
| cookies | Cookie | 0 | 不变 |
| scripts | 脚本 | 0 | 不变 |
| saved_responses | 保存的响应 | 0 | 不变 |
| chat_sessions | 聊天会话 | 0 | **拆分 messages → chat_messages 表** |
| chat_messages | 聊天消息 | 0 | **新建** |
| orchestrations | 编排 | 2 (steps, schedule) | 不变 |
| orchestration_runs | 编排执行 | 1 (steps) | 不变 |
| stress_configs | 压测配置 | 1 (assertions) | 不变 |
| stress_results | 压测结果 | 3 (config, status_distribution, error_distribution) | **拆分 failed_requests → stress_result_details** |
| stress_result_details | 失败请求详情 | 0 | **新建** |
| stress_result_points | 时序数据点 | 0 | **新建** |
| docs | API 文档 | 0 | 不变 |
| app_state | UI 状态 | 3 (expanded_ids, open_tabs, request_tabs) | 不变 |

> **JSON 列总数不变 (~25)，仅新增 3 张关系表**（chat_messages, stress_result_details, stress_result_points），解决真正需要分页查询的场景。

---

## 五、连接管理

### 5.1 设计思路

SQLite 本身是单写串行的，桌面应用也只有一个用户。r2d2 连接池带来的额外复杂度和依赖与收益不成正比。采用更轻量的方案：**每个工作区保持一个长连接，切换工作区时切换连接**。

### 5.2 单例长连接

```rust
use rusqlite::Connection;
use std::path::PathBuf;

pub struct WorkspaceConnection {
    pub conn: Connection,
    pub workspace_id: String,
}

impl WorkspaceConnection {
    pub fn open(workspace_id: &str, base_dir: &PathBuf) -> Result<Self, String> {
        let ws_dir = base_dir.join(workspace_id);
        std::fs::create_dir_all(&ws_dir)
            .map_err(|e| format!("创建工作区目录失败: {}", e))?;

        let db_path = ws_dir.join("data.db");
        let conn = Connection::open_with_flags(
            &db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        ).map_err(|e| format!("打开数据库失败: {}", e))?;

        // 一次性 PRAGMA 设置
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             PRAGMA synchronous=NORMAL;"
        ).map_err(|e| format!("设置 PRAGMA 失败: {}", e))?;

        Ok(Self {
            conn,
            workspace_id: workspace_id.to_string(),
        })
    }
}

/// 全局连接管理器（整个应用生命周期内保持一个活动连接）
pub struct ConnectionManager {
    current: Option<WorkspaceConnection>,
    base_dir: PathBuf,
}

impl ConnectionManager {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { current: None, base_dir }
    }

    /// 切换到指定工作区，关闭旧连接，打开新连接
    pub fn switch(&mut self, workspace_id: &str) -> Result<&Connection, String> {
        // 如果已经是同一个工作区，直接返回
        if let Some(ref c) = self.current {
            if c.workspace_id == workspace_id {
                return Ok(&c.conn);
            }
        }

        // 关闭旧连接（drop 会触发）
        self.current = None;

        // 打开新连接
        let ws = WorkspaceConnection::open(workspace_id, &self.base_dir)?;
        self.current = Some(ws);
        Ok(&self.current.as_ref().unwrap().conn)
    }

    /// 获取当前活动连接
    pub fn conn(&self) -> Option<&Connection> {
        self.current.as_ref().map(|c| &c.conn)
    }

    /// 关闭当前连接
    pub fn close(&mut self) {
        self.current = None;
    }
}
```

### 5.3 仓储使用方式

```rust
// 仓储不再持有连接，改为每次调用时传入
pub struct SqliteCollectionRepository;

impl SqliteCollectionRepository {
    pub fn read_all(conn: &Connection) -> Result<Vec<Collection>, String> {
        // ... 使用传入的 conn
    }
}

// 应用服务层持有 ConnectionManager，每次操作传 conn
impl CollectionApplicationService {
    pub fn get_collections(&self) -> Result<Vec<Collection>, String> {
        let conn = self.conn_mgr.conn().ok_or("未连接工作区")?;
        SqliteCollectionRepository::read_all(conn)
    }
}
```

### 5.4 对比

| 维度 | 旧方案（每次新建连接） | 单例长连接 | 连接池 (r2d2) |
|------|---------------------|-----------|-------------|
| 连接开销 | 每次 0.1-3ms | 仅切换工作区时 | 几乎为零 |
| 额外依赖 | 无 | 无 | 2 个 crate |
| 代码复杂度 | 最低 | 低 | 中等 |
| 并发控制 | 无 | 无（单用户无必要） | 有 |
| 文件句柄 | 可能泄漏 | 明确可控 | 池管理 |

---

## 六、全局配置 (config.json)

### 6.1 配置结构（保留不变）

```json
{
  "settings": {
    "request_timeout": 30000,
    "language": "zh-CN",
    "ai": {
      "api_endpoint": "",
      "encrypted_api_key": "",
      "model": "",
      "custom_headers": {},
      "timeout": 60000
    }
  },
  "workspaces": [
    {
      "id": "uuid-1",
      "name": "我的工作区",
      "description": "",
      "created_at": "2026-01-01T00:00:00Z",
      "last_opened": "2026-06-15T00:00:00Z"
    }
  ],
  "last_workspace_id": "uuid-1"
}
```

### 6.2 读写方式（保留不变）

- `JsonAppConfigRepository` 保持不变
- 路径改为: `{APP_DATA}/config.json`

---

## 七、安全性设计

### 7.1 加密密钥管理

```
{APP_DATA}/.keystore
  内部格式: encrypted_master_key#fingerprint
```

- **Master Key**: 随机生成 256-bit
- **派生密钥**: PBKDF2(machine_fingerprint + app_salt, 100_000 轮)
- **machine_fingerprint**: hostname + OS + 用户名
- **app_salt**: 编译时嵌入

### 7.2 敏感字段

- `config.json` → `settings.ai.encrypted_api_key` — AES-256-GCM 加密

---

## 八、性能优化清单

| 优化项 | 说明 | 优先级 |
|--------|------|--------|
| WAL 模式 | 读写并发提升 | P0 |
| 单例长连接 | 切换工作区时复用连接，避免频繁打开 | P0 |
| 外键 + 高频列索引 | 每个子表对应外键索引 | P0 |
| 预编译语句缓存 | `conn.prepare_cached()` | P1 |
| synchronous=NORMAL | WAL 下安全，性能 2-10x | P1 |
| cache_size | 默认 -2000 KB | P1 |
| 分页查询 | 历史记录、编排执行记录、压测结果 | P1 |
| 定期 VACUUM | 每月或手动 | P2 |

---

## 九、实现路线图

### Phase 1: 基础设施 (1-2 天)
- [ ] 实现 `ConnectionManager` 单例长连接管理器
- [ ] 编写初始 Schema DDL

### Phase 2: 仓储实现 (2-3 天)
- [ ] 实现 chat_messages、stress_result_details、stress_result_points 三张新表的 CRUD
- [ ] 更新 chat、stress 仓储适配新表
- [ ] 其余仓储保持 JSON 列读写不变，适配新连接管理

### Phase 3: 适配与清理 (1-2 天)
- [ ] 应用服务层适配新仓储接口
- [ ] `JsonAppConfigRepository` 路径切换到新目录
- [ ] 清理旧代码

### Phase 4: 测试 (1 天)
- [ ] 回归测试

---

## 十、对比总结

| 维度 | 旧方案 | 新方案 | 收益 |
|------|--------|--------|------|
| **全局配置** | config.json | config.json | 保留不变 |
| **工作区隔离** | 每工作区一个 db | 每工作区一个 db | 保留不变 |
| **文本内容** | 存 DB TEXT 列 | 存 DB TEXT 列 | 保留不变 |
| **DB 路径** | `./data/data_<id>.db` | `./data/<id>/data.db` | 子目录隔离 |
| **表数量** | 15 | 19 | 新增 3 张关系表 |
| **JSON 列数** | ~25 | ~25 | 不变（列表数据保留 JSON） |
| **关系表** | 几乎无 | 3 (chat_messages, stress_result_details, stress_result_points) | 分页查询场景 |
| **连接管理** | 每次新建连接 | 单例长连接 | 减少文件打开，切换工作区时复用 |
| **数据目录** | exe 同级 data/ | exe 同级 data/ | 不变 |

---

## 附录 A: 依赖变更 (Cargo.toml)

```toml
# 无需新增依赖

# 保留不变
rusqlite = { version = "0.31", features = ["bundled"] }
serde = "1"
serde_json = "1"
aes-gcm = "0.10"
base64 = "0.22"
sha2 = "0.10"
```

## 附录 B: 关键 SQL 对比

| 查询 | 旧方案 | 新方案 |
|------|--------|--------|
| 加载集合树 | 1 次全表扫描 + 7 个 JSON 列解析 | 相同（不变） |
| 查询某 API 的 Headers | JSON 反序列化 headers_json | 相同（不变） |
| 加载聊天消息 | 反序列化 messages_json 整列 | `SELECT * FROM chat_messages WHERE session_id = ? ORDER BY created_at` (分页) |
| 加载压测失败详情 | 反序列化 failed_requests_json 整列 | `SELECT * FROM stress_result_details WHERE result_id = ? ORDER BY request_index` (分页) |
| 加载压测时序数据 | 反序列化 history_points_json 整列 | `SELECT * FROM stress_result_points WHERE result_id = ? ORDER BY timestamp_ms` (分页) |

## 附录 C: 仓储实现清单

| 仓储 | 涉及表 | 变化 |
|------|--------|------|
| SqliteWorkspaceDataRepository | environments, cookies, app_state | 不变 |
| SqliteCollectionRepository | collections | 不变 |
| SqliteHistoryRepository | history_entries | 不变 |
| SqliteScriptRepository | scripts | 不变 |
| SqliteResponseRepository | saved_responses | 不变 |
| SqliteChatRepository | chat_sessions, chat_messages | **新增 chat_messages 子表** |
| SqliteOrchestrationRepository | orchestrations, orchestration_runs | 不变 |
| SqliteStressRepository | stress_configs, stress_results, stress_result_details, stress_result_points | **新增 2 张子表** |
| SqliteMdRepository | docs | 不变 |