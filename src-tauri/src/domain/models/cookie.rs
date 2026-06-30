use serde::{Deserialize, Serialize};

/// Cookie 数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u64>,
    pub secure: bool,
    pub http_only: bool,
    pub created_at: String,
}

/// Cookie 存储配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CookiesConfig {
    pub cookies: Vec<Cookie>,
}
