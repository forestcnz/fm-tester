use serde::{Deserialize, Serialize};

/// 脚本类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScriptTargetType {
    Api,
    Collection,
    Workspace,
    Environment,
}

/// 脚本种类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScriptKind {
    Pre,
    Post,
}

/// 脚本索引条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptIndexEntry {
    /// 目标类型（api/collection/workspace）
    pub target_type: ScriptTargetType,
    /// 目标 ID（api 或 collection 的 id，workspace 时为空）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    /// 脚本种类（pre/post）
    pub script_kind: ScriptKind,
    /// 脚本文件路径（相对于工作区）
    pub file: String,
}

/// 脚本索引配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScriptsConfig {
    pub scripts: Vec<ScriptIndexEntry>,
}
