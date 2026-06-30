//! SSE (Server-Sent Events) 客户端实现
//!
//! 提供 SSE 流式连接和事件接收功能。
//! 符合 DDD 架构规范：领域层定义接口，基础设施层实现。

use crate::domain::models::{Header, SseEvent, SseState, SseStatus};
use futures_util::StreamExt;
use hyper::body::HttpBody;
use reqwest;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

/// SSE 客户端服务
pub struct SseClientService {
    /// 取消信号
    cancelled: Arc<AtomicBool>,
}

impl SseClientService {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 处理已有的 SSE 响应体流（从 HTTP 请求检测到 SSE 后调用）
    pub async fn process_sse_stream(
        &self,
        app: AppHandle,
        mut body: hyper::Body,
    ) -> Result<(), String> {
        // 重置取消信号
        self.cancelled.store(false, Ordering::SeqCst);

        let connected_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // 发送已连接状态
        app.emit(
            "sse-state",
            SseState {
                status: SseStatus::Connected,
                events: vec![],
                error: None,
                connected_at,
            },
        )
        .ok();

        // 处理流式响应
        let mut buffer = String::new();
        let mut current_event_id: Option<String> = None;
        let mut current_event_type: Option<String> = None;
        let mut current_data_lines: Vec<String> = vec![];

        while let Some(chunk) = body.data().await {
            // 检查是否被取消
            if self.cancelled.load(Ordering::SeqCst) {
                app.emit(
                    "sse-state",
                    SseState {
                        status: SseStatus::Disconnected,
                        events: vec![],
                        error: None,
                        connected_at,
                    },
                )
                .ok();
                return Ok(());
            }

            let chunk = chunk.map_err(|e| {
                app.emit(
                    "sse-state",
                    SseState {
                        status: SseStatus::Error,
                        events: vec![],
                        error: Some(format!("流错误: {}", e)),
                        connected_at,
                    },
                )
                .ok();
                format!("流错误: {}", e)
            })?;
            let chunk_text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_text);

            // 解析 SSE 格式
            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                // 空行：事件结束，发送事件
                if line.is_empty() {
                    if !current_data_lines.is_empty() {
                        // 构建事件
                        let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64;

                        let event = SseEvent {
                            id: current_event_id.clone(),
                            event: current_event_type.clone().or(Some("message".to_string())),
                            data: current_data_lines.join("\n"),
                            timestamp,
                        };

                        // 发送事件
                        app.emit("sse-event", event).ok();

                        // 重置当前事件
                        current_event_id = None;
                        current_event_type = None;
                        current_data_lines = vec![];
                    }
                } else if line.contains(':') {
                    let colon_pos = line.find(':').unwrap();
                    let field_name = &line[..colon_pos];
                    let field_value = &line[colon_pos + 1..];

                    // 处理冒号后的空格（SSE 规范允许）
                    let field_value = field_value.strip_prefix(' ').unwrap_or(field_value);

                    match field_name {
                        "id" => current_event_id = Some(field_value.to_string()),
                        "event" => current_event_type = Some(field_value.to_string()),
                        "data" => current_data_lines.push(field_value.to_string()),
                        // 忽略其他字段（retry 等）
                        _ => {}
                    }
                } else if !line.starts_with(':') {
                    // 没有冒号的行：视为 data 字段（不带字段名）
                    current_data_lines.push(line);
                }
                // 以冒号开头的行是注释，忽略
            }
        }

        // 流结束，发送断开状态
        app.emit(
            "sse-state",
            SseState {
                status: SseStatus::Disconnected,
                events: vec![],
                error: None,
                connected_at,
            },
        )
        .ok();

        Ok(())
    }

    /// 开始 SSE 连接（独立发送请求）
    pub async fn start_sse(
        &self,
        app: AppHandle,
        method: &str,
        url: &str,
        headers: Vec<Header>,
        body: Option<String>,
        timeout_ms: u64,
        last_event_id: Option<String>,
    ) -> Result<(), String> {
        // 重置取消信号
        self.cancelled.store(false, Ordering::SeqCst);

        // 发送连接状态
        let connected_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        app.emit(
            "sse-state",
            SseState {
                status: SseStatus::Connecting,
                events: vec![],
                error: None,
                connected_at,
            },
        )
        .ok();

        // 创建 HTTP 客户端
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .danger_accept_invalid_certs(false)
            .build()
            .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

        // 构建请求（支持 GET 或 POST）
        let method_upper = method.to_uppercase();
        let mut request = if method_upper == "POST" {
            client.post(url)
        } else {
            client.get(url)
        };

        // 如果有 body，添加请求体
        if let Some(body_content) = body {
            request = request.body(body_content);
            // 如果没有显式设置 Content-Type，默认为 application/json
            let has_content_type = headers
                .iter()
                .any(|h| h.key.to_lowercase() == "content-type");
            if !has_content_type {
                request = request.header("Content-Type", "application/json");
            }
        }

        // 添加自定义 headers
        for header in &headers {
            if header.enabled && !header.key.trim().is_empty() {
                request = request.header(&header.key, &header.value);
            }
        }

        // 添加 Last-Event-ID header（用于重连）
        if let Some(id) = last_event_id {
            request = request.header("Last-Event-ID", id);
        }

        // 发送请求
        let response = request.send().await.map_err(|e| {
            app.emit(
                "sse-state",
                SseState {
                    status: SseStatus::Error,
                    events: vec![],
                    error: Some(format!("连接失败: {}", e)),
                    connected_at,
                },
            )
            .ok();
            format!("连接失败: {}", e)
        })?;

        // 检查响应状态
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            app.emit(
                "sse-state",
                SseState {
                    status: SseStatus::Error,
                    events: vec![],
                    error: Some(format!("HTTP 错误 {}: {}", status, body)),
                    connected_at,
                },
            )
            .ok();
            return Err(format!("HTTP 错误 {}: {}", status, body));
        }

        // 发送已连接状态
        app.emit(
            "sse-state",
            SseState {
                status: SseStatus::Connected,
                events: vec![],
                error: None,
                connected_at,
            },
        )
        .ok();

        // 处理流式响应
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut current_event_id: Option<String> = None;
        let mut current_event_type: Option<String> = None;
        let mut current_data_lines: Vec<String> = vec![];

        while let Some(chunk) = stream.next().await {
            // 检查是否被取消
            if self.cancelled.load(Ordering::SeqCst) {
                app.emit(
                    "sse-state",
                    SseState {
                        status: SseStatus::Disconnected,
                        events: vec![],
                        error: None,
                        connected_at,
                    },
                )
                .ok();
                return Ok(());
            }

            let chunk = chunk.map_err(|e| {
                app.emit(
                    "sse-state",
                    SseState {
                        status: SseStatus::Error,
                        events: vec![],
                        error: Some(format!("流错误: {}", e)),
                        connected_at,
                    },
                )
                .ok();
                format!("流错误: {}", e)
            })?;
            let chunk_text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_text);

            // 解析 SSE 格式
            // SSE 格式规范：每行以 \n 结束，空行表示事件结束
            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                // 空行：事件结束，发送事件
                if line.is_empty() {
                    if !current_data_lines.is_empty() {
                        // 构建事件
                        let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64;

                        let event = SseEvent {
                            id: current_event_id.clone(),
                            event: current_event_type.clone().or(Some("message".to_string())),
                            data: current_data_lines.join("\n"),
                            timestamp,
                        };

                        // 发送事件到前端
                        app.emit("sse-event", event.clone()).ok();

                        // 清空当前事件数据
                        current_event_id = None;
                        current_event_type = None;
                        current_data_lines.clear();
                    }
                    continue;
                }

                // 解析字段
                if let Some(colon_pos) = line.find(':') {
                    let field_name = &line[..colon_pos];
                    let field_value = &line[colon_pos + 1..];

                    // 处理冒号后的空格（SSE 规范允许）
                    let field_value = field_value.strip_prefix(' ').unwrap_or(field_value);

                    match field_name {
                        "id" => current_event_id = Some(field_value.to_string()),
                        "event" => current_event_type = Some(field_value.to_string()),
                        "data" => current_data_lines.push(field_value.to_string()),
                        // 忽略其他字段（retry 等）
                        _ => {}
                    }
                } else if !line.starts_with(':') {
                    // 没有冒号的行：视为 data 字段（不带字段名）
                    current_data_lines.push(line);
                }
                // 以冒号开头的行是注释，忽略
            }
        }

        // 流结束，发送断开状态
        app.emit(
            "sse-state",
            SseState {
                status: SseStatus::Disconnected,
                events: vec![],
                error: None,
                connected_at,
            },
        )
        .ok();

        Ok(())
    }

    /// 停止 SSE 连接
    pub fn stop_sse(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}

impl Default for SseClientService {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局 SSE 客户端单例
///
/// 注意：必须与 `interface::commands::sse::SSE_CLIENT` 单例保持一致，
/// 否则 stop 命令操作的不是处理流时使用的实例，会导致无法停止。
/// 此处使用 OnceLock 提供统一的全局访问点，所有调用方（包括 http_service
/// 检测到 text/event-stream 后转 SSE 流的场景）都使用同一实例。
static GLOBAL_SSE_CLIENT: std::sync::OnceLock<SseClientService> = std::sync::OnceLock::new();

/// 获取全局 SSE 客户端单例
pub fn get_sse_client() -> &'static SseClientService {
    GLOBAL_SSE_CLIENT.get_or_init(SseClientService::new)
}
