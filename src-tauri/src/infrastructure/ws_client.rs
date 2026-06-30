//! WebSocket 客户端实现
//!
//! 提供 WebSocket 连接、消息收发、心跳、重连等功能。
//! 符合 DDD 架构规范：基础设施层实现技术细节。

use crate::domain::models::{WsHeader, WsMessage, WsParam, WsState, WsStatus};
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio_tungstenite::{
    connect_async,
    tungstenite::client::IntoClientRequest,
    tungstenite::http::{HeaderName, HeaderValue},
    tungstenite::protocol::Message,
};

/// WebSocket 客户端服务（全局单例）
///
/// 使用 Arc<Mutex> 管理连接状态，支持单连接模式。
pub struct WsClientService {
    /// 取消信号（断开连接）
    cancelled: Arc<AtomicBool>,
    /// WebSocket 连接（可选）
    connection: Arc<Mutex<Option<WsConnection>>>,
}

/// WebSocket 连接封装
struct WsConnection {
    /// WebSocket 发送端
    sender: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
}

impl WsClientService {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            connection: Arc::new(Mutex::new(None)),
        }
    }

    /// 建立 WebSocket 连接
    ///
    /// # 参数
    /// - `app`: Tauri AppHandle，用于发送事件
    /// - `url`: WebSocket URL
    /// - `headers`: 连接请求头
    /// - `params`: URL 参数
    ///
    /// # 返回
    /// - 成功：连接成功
    /// - 失败：错误信息
    pub async fn connect(
        &self,
        app: AppHandle,
        url: &str,
        headers: Vec<WsHeader>,
        params: Vec<WsParam>,
    ) -> Result<(), String> {
        // 重置取消信号
        self.cancelled.store(false, Ordering::SeqCst);

        // 发送连接状态
        let connected_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        app.emit(
            "ws-state",
            WsState {
                status: WsStatus::Connecting,
                error: None,
                connected_at: None,
            },
        )
        .ok();

        // 构建完整 URL（添加参数）
        let full_url = self.build_url_with_params(url, params)?;

        // 构建带自定义请求头的客户端请求
        // IntoClientRequest 会自动补齐 Host / Connection / Upgrade / Sec-WebSocket-* 必需头
        let mut request = full_url.as_str().into_client_request().map_err(|e| {
            app.emit(
                "ws-state",
                WsState {
                    status: WsStatus::Error,
                    error: Some(format!("构建请求失败: {}", e)),
                    connected_at: None,
                },
            )
            .ok();
            format!("构建请求失败: {}", e)
        })?;

        // 追加用户自定义请求头（如 Authorization、Cookie 等）
        // 用户传的合法头会覆盖默认值（insert 是替换语义）
        {
            let req_headers = request.headers_mut();
            for header in &headers {
                if !header.enabled || header.key.trim().is_empty() {
                    continue;
                }
                let name = match HeaderName::from_bytes(header.key.as_bytes()) {
                    Ok(n) => n,
                    Err(_) => continue, // 跳过非法 header 名
                };
                let value = match HeaderValue::from_str(&header.value) {
                    Ok(v) => v,
                    Err(_) => continue, // 跳过非法 header 值
                };
                req_headers.insert(name, value);
            }
        }

        // 建立 WebSocket 连接
        let (ws_stream, _) = connect_async(request).await.map_err(|e| {
            app.emit(
                "ws-state",
                WsState {
                    status: WsStatus::Error,
                    error: Some(format!("连接失败: {}", e)),
                    connected_at: None,
                },
            )
            .ok();
            format!("连接失败: {}", e)
        })?;

        // 分离读写流
        let (sender, receiver) = ws_stream.split();

        // 存储连接
        let ws_conn = WsConnection { sender };

        {
            let mut conn = self.connection.lock().await;
            *conn = Some(ws_conn);
        }

        // 发送已连接状态
        app.emit(
            "ws-state",
            WsState {
                status: WsStatus::Connected,
                error: None,
                connected_at: Some(connected_at),
            },
        )
        .ok();

        // 启动消息接收循环
        self.handle_messages(app, receiver);

        Ok(())
    }

    /// 构建带参数的 URL
    fn build_url_with_params(&self, url: &str, params: Vec<WsParam>) -> Result<String, String> {
        if params.is_empty() {
            return Ok(url.to_string());
        }

        // 过滤启用的参数
        let enabled_params: Vec<&WsParam> = params.iter().filter(|p| p.enabled).collect();

        if enabled_params.is_empty() {
            return Ok(url.to_string());
        }

        // 构建 query string
        let query_parts: Vec<String> = enabled_params
            .iter()
            .map(|p| {
                let encoded_key = urlencoding::encode(&p.key);
                let encoded_value = urlencoding::encode(&p.value);
                format!("{}={}", encoded_key, encoded_value)
            })
            .collect();

        let query_string = query_parts.join("&");

        // 检查 URL 是否已有 query
        if url.contains('?') {
            Ok(format!("{}&{}", url, query_string))
        } else {
            Ok(format!("{}?{}", url, query_string))
        }
    }

    /// 启动消息接收循环
    fn handle_messages(
        &self,
        app: AppHandle,
        mut receiver: futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
    ) {
        let cancelled = self.cancelled.clone();
        let connection = self.connection.clone();

        tokio::spawn(async move {
            while let Some(message) = receiver.next().await {
                // 检查是否被取消
                if cancelled.load(Ordering::SeqCst) {
                    break;
                }

                match message {
                    Ok(msg) => {
                        match msg {
                            Message::Text(text) => {
                                // 创建接收消息
                                let ws_msg = WsMessage::new_received(text, "text".to_string());

                                // 发送消息事件
                                app.emit("ws-message", ws_msg.clone()).ok();
                            }
                            Message::Binary(data) => {
                                // 将二进制数据转换为 Base64
                                use base64::{engine::general_purpose::STANDARD, Engine as _};
                                let base64_content = STANDARD.encode(&data);

                                // 创建接收消息
                                let ws_msg =
                                    WsMessage::new_received(base64_content, "binary".to_string());

                                // 发送消息事件
                                app.emit("ws-message", ws_msg.clone()).ok();
                            }
                            Message::Ping(data) => {
                                // 自动回复 Pong（tungstenite 会自动处理）
                                // 这里仅记录日志，不发送事件
                                app.emit(
                                    "ws-message",
                                    WsMessage::new_received(
                                        format!("收到 Ping: {} bytes", data.len()),
                                        "text".to_string(),
                                    ),
                                )
                                .ok();
                            }
                            Message::Pong(data) => {
                                // 记录 Pong 消息
                                app.emit(
                                    "ws-message",
                                    WsMessage::new_received(
                                        format!("收到 Pong: {} bytes", data.len()),
                                        "text".to_string(),
                                    ),
                                )
                                .ok();
                            }
                            Message::Close(frame) => {
                                // 服务器主动关闭连接
                                let close_msg = if let Some(frame) = frame {
                                    format!("连接已关闭: {} (code: {})", frame.reason, frame.code)
                                } else {
                                    "连接已关闭".to_string()
                                };

                                app.emit(
                                    "ws-state",
                                    WsState {
                                        status: WsStatus::Disconnected,
                                        error: None,
                                        connected_at: None,
                                    },
                                )
                                .ok();

                                app.emit(
                                    "ws-message",
                                    WsMessage::new_received(close_msg, "text".to_string()),
                                )
                                .ok();

                                // 清空连接
                                let mut conn = connection.lock().await;
                                *conn = None;

                                break;
                            }
                            Message::Frame(_) => {
                                // 原始帧，通常不处理
                            }
                        }
                    }
                    Err(e) => {
                        // 连接错误
                        app.emit(
                            "ws-state",
                            WsState {
                                status: WsStatus::Error,
                                error: Some(format!("消息接收错误: {}", e)),
                                connected_at: None,
                            },
                        )
                        .ok();

                        // 清空连接
                        let mut conn = connection.lock().await;
                        *conn = None;

                        break;
                    }
                }
            }
            // 注意：不在这里发送 Disconnected 状态，避免重复
            // - 服务器关闭：已在 Message::Close 处理中发送
            // - 用户断开：已在 disconnect 函数中发送
            // - 错误断开：已在 Err 处理中发送
        });
    }

    /// 发送消息
    ///
    /// # 参数
    /// - `content`: 消息内容
    /// - `message_type`: 消息类型 ("text" 或 "binary")
    pub async fn send_message(
        &self,
        app: AppHandle,
        content: &str,
        message_type: &str,
    ) -> Result<(), String> {
        // 获取连接
        let mut conn = self.connection.lock().await;

        if let Some(ref mut ws_conn) = *conn {
            // 创建消息
            let message = match message_type {
                "binary" => {
                    // 解码 Base64 为二进制
                    use base64::{engine::general_purpose::STANDARD, Engine as _};
                    let data = STANDARD
                        .decode(content)
                        .map_err(|e| format!("Base64 解码失败: {}", e))?;
                    Message::Binary(data)
                }
                _ => Message::Text(content.to_string()),
            };

            // 发送消息
            ws_conn
                .sender
                .send(message)
                .await
                .map_err(|e| format!("发送失败: {}", e))?;

            // 创建发送消息记录
            let ws_msg = WsMessage::new_sent(content.to_string(), message_type.to_string());

            // 发送消息事件（让前端知道已发送）
            app.emit("ws-message", ws_msg).ok();

            Ok(())
        } else {
            Err("WebSocket 未连接".to_string())
        }
    }

    /// 断开连接
    pub async fn disconnect(&self, app: AppHandle) -> Result<(), String> {
        // 设置取消信号
        self.cancelled.store(true, Ordering::SeqCst);

        // 获取连接并发送 Close 帧
        let mut conn = self.connection.lock().await;

        if let Some(ref mut ws_conn) = *conn {
            // 发送 Close 帧
            ws_conn
                .sender
                .send(Message::Close(None))
                .await
                .map_err(|e| format!("发送 Close 失败: {}", e))?;

            // 清空连接
            *conn = None;
        }

        // 发送断开状态（只在主动断开时发送，避免与 handle_messages 重复）
        app.emit(
            "ws-state",
            WsState {
                status: WsStatus::Disconnected,
                error: None,
                connected_at: None,
            },
        )
        .ok();

        Ok(())
    }

    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        let conn = self.connection.lock().await;
        conn.is_some()
    }
}

impl Default for WsClientService {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    static ref WS_CLIENT: Arc<WsClientService> = Arc::new(WsClientService::new());
}

/// 获取全局 WebSocket 客户端实例
pub fn get_ws_client() -> Arc<WsClientService> {
    WS_CLIENT.clone()
}
