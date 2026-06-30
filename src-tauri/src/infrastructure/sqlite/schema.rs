//! SQLite Schema 定义
//!
//! 精简版数据库结构，将列表数据序列化为 JSON 存储。
//! 仅拆分大数据量的关系表（chat_messages、stress_result_details、stress_result_points）。

/// 完整的数据库 schema SQL
pub const SCHEMA_SQL: &str = "
-- 环境（variables 和 common_headers 存为 JSON）
CREATE TABLE IF NOT EXISTS environments (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    variables_json TEXT NOT NULL DEFAULT '[]',
    common_headers_json TEXT NOT NULL DEFAULT '[]',
    order_index INTEGER NOT NULL DEFAULT 0
);

-- 应用状态（合并所有 memory 表 + 激活环境）
CREATE TABLE IF NOT EXISTS app_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    expanded_ids_json TEXT NOT NULL DEFAULT '[]',
    open_tabs_json TEXT NOT NULL DEFAULT '[]',
    active_tab_index INTEGER NOT NULL DEFAULT 0,
    request_tabs_json TEXT NOT NULL DEFAULT '{}',
    active_environment_id TEXT REFERENCES environments(id) ON DELETE SET NULL
);

-- WebSocket 配置
CREATE TABLE IF NOT EXISTS ws_configs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    headers_json TEXT NOT NULL DEFAULT '[]',
    params_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 0
);

-- Cookie
CREATE TABLE IF NOT EXISTS cookies (
    name TEXT NOT NULL,
    domain TEXT NOT NULL,
    path TEXT NOT NULL DEFAULT '/',
    value TEXT NOT NULL DEFAULT '',
    expires TEXT,
    max_age INTEGER,
    secure INTEGER NOT NULL DEFAULT 0,
    http_only INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    PRIMARY KEY (name, domain)
);

CREATE INDEX IF NOT EXISTS idx_cookie_domain ON cookies(domain);

-- 集合项（params/headers/form_fields/variables/common_headers 存为 JSON）
CREATE TABLE IF NOT EXISTS collection_items (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    item_type TEXT NOT NULL,
    parent_id TEXT,
    order_index INTEGER NOT NULL DEFAULT 0,
    method TEXT,
    url TEXT,
    body TEXT,
    body_type TEXT,
    params_json TEXT NOT NULL DEFAULT '[]',
    headers_json TEXT NOT NULL DEFAULT '[]',
    form_fields_json TEXT NOT NULL DEFAULT '[]',
    form_files_json TEXT NOT NULL DEFAULT '[]',
    common_headers_json TEXT NOT NULL DEFAULT '[]',
    variables_json TEXT NOT NULL DEFAULT '[]',
    saved_response_ids_json TEXT NOT NULL DEFAULT '[]'
);

CREATE INDEX IF NOT EXISTS idx_item_parent ON collection_items(parent_id);
CREATE INDEX IF NOT EXISTS idx_item_type ON collection_items(item_type);

-- 历史记录（headers 存为 JSON）
CREATE TABLE IF NOT EXISTS history_entries (
    id TEXT PRIMARY KEY,
    method TEXT NOT NULL,
    url TEXT NOT NULL,
    resolved_url TEXT NOT NULL,
    status INTEGER NOT NULL,
    status_text TEXT NOT NULL,
    response_body TEXT NOT NULL DEFAULT '',
    time INTEGER NOT NULL DEFAULT 0,
    size INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    body TEXT,
    body_type TEXT,
    api_id TEXT,
    api_name TEXT,
    date TEXT NOT NULL,
    request_headers_json TEXT NOT NULL DEFAULT '[]',
    response_headers_json TEXT NOT NULL DEFAULT '[]',
    form_fields_json TEXT NOT NULL DEFAULT '[]'
);

CREATE INDEX IF NOT EXISTS idx_history_date ON history_entries(date);
CREATE INDEX IF NOT EXISTS idx_history_api ON history_entries(api_id);
CREATE INDEX IF NOT EXISTS idx_history_created ON history_entries(created_at);
CREATE INDEX IF NOT EXISTS idx_history_url ON history_entries(url);

-- 脚本
CREATE TABLE IF NOT EXISTS scripts (
    id TEXT PRIMARY KEY,
    target_type TEXT NOT NULL,
    target_id TEXT,
    script_kind TEXT NOT NULL,
    filename TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT ''
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_script_target ON scripts(target_type, target_id, script_kind);

-- 保存响应（仅保存基本信息和 MD 文档）
CREATE TABLE IF NOT EXISTS saved_responses (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    api_id TEXT,
    doc_content TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_saved_resp_api ON saved_responses(api_id);

-- Chat 会话
CREATE TABLE IF NOT EXISTS chat_sessions (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    title TEXT,
    active_session INTEGER NOT NULL DEFAULT 0
);

-- Chat 消息（拆分自原 messages_json 列，支持分页查询）
CREATE TABLE IF NOT EXISTS chat_messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    reasoning TEXT,
    timestamp TEXT
);

CREATE INDEX IF NOT EXISTS idx_chat_msg_session ON chat_messages(session_id);

-- 编排（steps/schedule 存为 JSON）
CREATE TABLE IF NOT EXISTS orchestrations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 0,
    steps_json TEXT NOT NULL DEFAULT '[]',
    schedule_json TEXT NOT NULL DEFAULT '{}'
);

-- 编排执行记录（step_results 存为 JSON）
CREATE TABLE IF NOT EXISTS orchestration_runs (
    id TEXT PRIMARY KEY,
    orchestration_id TEXT NOT NULL REFERENCES orchestrations(id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    total_time INTEGER NOT NULL DEFAULT 0,
    success_count INTEGER NOT NULL DEFAULT 0,
    failed_count INTEGER NOT NULL DEFAULT 0,
    skipped_count INTEGER NOT NULL DEFAULT 0,
    steps_json TEXT NOT NULL DEFAULT '[]'
);

CREATE INDEX IF NOT EXISTS idx_orch_run_orch ON orchestration_runs(orchestration_id);

-- 压测配置（assertions 存为 JSON）
CREATE TABLE IF NOT EXISTS stress_configs (
    api_id TEXT PRIMARY KEY,
    concurrent INTEGER NOT NULL DEFAULT 10,
    total_requests INTEGER,
    duration_seconds INTEGER,
    ramp_up_seconds INTEGER NOT NULL DEFAULT 0,
    timeout_ms INTEGER NOT NULL DEFAULT 30000,
    assertions_json TEXT NOT NULL DEFAULT '[]'
);

-- 压测结果（聚合数据存 JSON，详情拆分为独立表）
CREATE TABLE IF NOT EXISTS stress_results (
    id TEXT PRIMARY KEY,
    api_id TEXT NOT NULL,
    config_json TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    total_requests INTEGER NOT NULL DEFAULT 0,
    successful_requests INTEGER NOT NULL DEFAULT 0,
    failed_requests INTEGER NOT NULL DEFAULT 0,
    total_time_ms INTEGER NOT NULL DEFAULT 0,
    qps REAL NOT NULL DEFAULT 0.0,
    avg_time_ms REAL NOT NULL DEFAULT 0.0,
    min_time_ms INTEGER NOT NULL DEFAULT 0,
    max_time_ms INTEGER NOT NULL DEFAULT 0,
    p50_time_ms REAL NOT NULL DEFAULT 0.0,
    p90_time_ms REAL NOT NULL DEFAULT 0.0,
    p95_time_ms REAL NOT NULL DEFAULT 0.0,
    p99_time_ms REAL NOT NULL DEFAULT 0.0,
    success_rate REAL NOT NULL DEFAULT 0.0,
    status_distribution_json TEXT NOT NULL DEFAULT '{}',
    error_distribution_json TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_stress_res_api ON stress_results(api_id);
CREATE INDEX IF NOT EXISTS idx_stress_res_time ON stress_results(start_time);
CREATE INDEX IF NOT EXISTS idx_stress_res_api_time ON stress_results(api_id, start_time);

-- 压测失败请求详情（拆分自原 failed_request_details 列，支持分页查询）
CREATE TABLE IF NOT EXISTS stress_result_details (
    id TEXT PRIMARY KEY,
    result_id TEXT NOT NULL REFERENCES stress_results(id) ON DELETE CASCADE,
    time TEXT NOT NULL,
    error TEXT NOT NULL DEFAULT '',
    status INTEGER,
    elapsed_ms INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_stress_detail_res ON stress_result_details(result_id);

-- 压测历史数据点（拆分自原 history 列，支持分页查询）
CREATE TABLE IF NOT EXISTS stress_result_points (
    id TEXT PRIMARY KEY,
    result_id TEXT NOT NULL REFERENCES stress_results(id) ON DELETE CASCADE,
    second INTEGER NOT NULL,
    qps REAL NOT NULL DEFAULT 0.0,
    avg_time_ms REAL NOT NULL DEFAULT 0.0,
    successful INTEGER NOT NULL DEFAULT 0,
    failed INTEGER NOT NULL DEFAULT 0,
    requests INTEGER NOT NULL DEFAULT 0,
    concurrent INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_stress_point_res ON stress_result_points(result_id);

-- 文档（合并 doc_index 和 doc_content）
CREATE TABLE IF NOT EXISTS docs (
    api_id TEXT PRIMARY KEY,
    updated_at TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT ''
);
";
