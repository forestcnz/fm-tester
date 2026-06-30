use crate::domain::models::{FormField, Header};
use serde::Serialize;

/// 解析后的 curl 命令结果
#[derive(Debug, Serialize, Clone)]
pub struct ParsedCurl {
    pub method: String,
    pub url: String,
    pub headers: Vec<Header>,
    pub body: Option<String>,
    pub body_type: Option<String>,
    pub form_fields: Option<Vec<FormField>>,
}
