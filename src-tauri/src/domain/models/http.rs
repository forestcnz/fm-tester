use super::Header;
use serde::{Deserialize, Serialize};

/// 请求耗时分解（各阶段毫秒）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestTiming {
    /// DNS 解析
    pub dns_ms: u64,
    /// TCP 连接
    pub connect_ms: u64,
    /// TLS 握手（仅 https）
    pub tls_ms: u64,
    /// 发送请求到收到首字节
    pub ttfb_ms: u64,
    /// 读取响应体
    pub download_ms: u64,
    /// 端到端总耗时
    pub total_ms: u64,
}

/// HTTP 响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
    pub time: u64,
    pub size: u64,
    pub resolved_url: String,
    pub resolved_headers: Vec<Header>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<RequestTiming>,
}
