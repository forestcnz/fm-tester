//! WebSocket 应用服务
//!
//! 处理 WebSocket 连接相关的业务逻辑，协调基础设施 WebSocket 客户端。
//! 包括变量替换、日志记录、连接管理、配置持久化等功能。

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::domain::models::common::generate_id;
use crate::domain::models::{WsConfigEntry, WsHeader, WsParam};
use crate::infrastructure::ws_client::get_ws_client;
use crate::infrastructure::RepositoryFactory;

/// WebSocket 日志结构
#[derive(Clone, Serialize)]
pub struct WsLog {
    #[serde(rename = "logType")]
    pub log_type: String,
    pub timestamp: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// WebSocket 应用服务
pub struct WsApplicationService;

impl WsApplicationService {
    /// 发送日志事件
    pub fn emit_log(app: &AppHandle, log: WsLog) {
        if let Err(e) = app.emit("ws-log", log) {
            eprintln!("发送 WebSocket 日志事件失败: {}", e);
        }
    }

    /// 建立 WebSocket 连接
    ///
    /// # 参数
    /// - `app`: Tauri AppHandle
    /// - `url`: WebSocket URL
    /// - `headers`: 连接请求头
    /// - `params`: URL 参数
    /// - `workspace_id`: 工作区 ID（用于变量替换）
    /// - `ws_id`: WebSocket 配置 ID（可选，用于消息历史）
    ///
    /// # 返回
    /// - 成功：连接成功
    /// - 失败：错误信息
    pub async fn connect_ws(
        app: AppHandle,
        url: String,
        headers: Vec<WsHeader>,
        params: Vec<WsParam>,
        workspace_id: String,
        ws_id: Option<String>,
    ) -> Result<(), String> {
        // 发送日志
        Self::emit_log(
            &app,
            WsLog {
                log_type: "info".to_string(),
                timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                message: format!("正在连接 WebSocket: {}", url),
                data: Some(serde_json::json!({
                    "url": url,
                    "workspace_id": workspace_id,
                    "ws_id": ws_id,
                })),
                error: None,
            },
        );

        // 获取 WebSocket 客户端
        let ws_client = get_ws_client();

        // 检查是否已连接
        if ws_client.is_connected().await {
            Self::emit_log(
                &app,
                WsLog {
                    log_type: "warn".to_string(),
                    timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    message: "已有 WebSocket 连接，先断开旧连接".to_string(),
                    data: None,
                    error: None,
                },
            );

            // 断开旧连接
            ws_client.disconnect(app.clone()).await?;
        }

        // 建立 WebSocket 连接
        let result = ws_client.connect(app.clone(), &url, headers, params).await;

        match &result {
            Ok(_) => {
                Self::emit_log(
                    &app,
                    WsLog {
                        log_type: "success".to_string(),
                        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        message: "WebSocket 连接成功".to_string(),
                        data: Some(serde_json::json!({
                            "url": url,
                        })),
                        error: None,
                    },
                );
            }
            Err(error) => {
                Self::emit_log(
                    &app,
                    WsLog {
                        log_type: "error".to_string(),
                        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        message: format!("WebSocket 连接失败: {}", error),
                        data: Some(serde_json::json!({
                            "url": url,
                        })),
                        error: Some(error.clone()),
                    },
                );
            }
        }

        result
    }

    /// 发送 WebSocket 消息
    ///
    /// # 参数
    /// - `app`: Tauri AppHandle
    /// - `content`: 消息内容
    /// - `message_type`: 消息类型 ("text" 或 "binary")
    ///
    /// # 返回
    /// - 成功：发送成功
    /// - 失败：错误信息
    pub async fn send_ws_message(
        app: AppHandle,
        content: String,
        message_type: String,
    ) -> Result<(), String> {
        // 发送日志
        Self::emit_log(
            &app,
            WsLog {
                log_type: "info".to_string(),
                timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                message: format!("发送 WebSocket 消息: {}", content),
                data: Some(serde_json::json!({
                    "content": content,
                    "type": message_type,
                })),
                error: None,
            },
        );

        // 获取 WebSocket 客户端
        let ws_client = get_ws_client();

        // 发送消息
        let result = ws_client
            .send_message(app.clone(), &content, &message_type)
            .await;

        match &result {
            Ok(_) => {
                Self::emit_log(
                    &app,
                    WsLog {
                        log_type: "success".to_string(),
                        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        message: "消息发送成功".to_string(),
                        data: None,
                        error: None,
                    },
                );
            }
            Err(error) => {
                Self::emit_log(
                    &app,
                    WsLog {
                        log_type: "error".to_string(),
                        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        message: format!("消息发送失败: {}", error),
                        data: None,
                        error: Some(error.clone()),
                    },
                );
            }
        }

        result
    }

    /// 断开 WebSocket 连接
    ///
    /// # 参数
    /// - `app`: Tauri AppHandle
    ///
    /// # 返回
    /// - 成功：断开成功
    /// - 失败：错误信息
    pub async fn disconnect_ws(app: AppHandle) -> Result<(), String> {
        // 发送日志
        Self::emit_log(
            &app,
            WsLog {
                log_type: "info".to_string(),
                timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                message: "正在断开 WebSocket 连接".to_string(),
                data: None,
                error: None,
            },
        );

        // 获取 WebSocket 客户端
        let ws_client = get_ws_client();

        // 断开连接
        let result = ws_client.disconnect(app.clone()).await;

        match &result {
            Ok(_) => {
                Self::emit_log(
                    &app,
                    WsLog {
                        log_type: "success".to_string(),
                        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        message: "WebSocket 已断开".to_string(),
                        data: None,
                        error: None,
                    },
                );
            }
            Err(error) => {
                Self::emit_log(
                    &app,
                    WsLog {
                        log_type: "error".to_string(),
                        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        message: format!("断开连接失败: {}", error),
                        data: None,
                        error: Some(error.clone()),
                    },
                );
            }
        }

        result
    }

    /// 检查 WebSocket 连接状态
    pub async fn is_connected() -> bool {
        let ws_client = get_ws_client();
        ws_client.is_connected().await
    }

    // ===== WebSocket 配置 CRUD（通过 Repository 持久化）=====

    /// 读取工作区下所有 WebSocket 配置
    pub fn list_ws_configs(workspace_id: &str) -> Result<Vec<WsConfigEntry>, String> {
        RepositoryFactory::get_ws_config_repository().list(workspace_id)
    }

    /// 保存（新建或更新）WebSocket 配置
    ///
    /// - 传入 `existing_id` 时更新该 id 的配置；若该 id 不存在则插入（修复了原 UPDATE 0 行不插入的 bug）
    /// - 不传入 `existing_id` 时生成新 id 并插入
    pub fn save_ws_config(
        workspace_id: &str,
        existing_id: Option<String>,
        name: String,
        url: String,
        headers: Vec<WsHeader>,
        params: Vec<WsParam>,
    ) -> Result<String, String> {
        let now = chrono::Local::now().to_rfc3339();
        let id = existing_id.unwrap_or_else(|| generate_id("ws"));

        // 读取现有配置以保留 created_at（更新场景）
        let created_at = RepositoryFactory::get_ws_config_repository()
            .list(workspace_id)
            .ok()
            .and_then(|configs| {
                configs
                    .into_iter()
                    .find(|c| c.id == id)
                    .map(|c| c.created_at)
            })
            .unwrap_or_else(|| now.clone());

        let entry = WsConfigEntry {
            id: id.clone(),
            name,
            url,
            headers,
            params,
            created_at,
            updated_at: now,
        };

        RepositoryFactory::get_ws_config_repository().upsert(workspace_id, &entry)?;
        Ok(id)
    }

    /// 删除 WebSocket 配置
    pub fn delete_ws_config(workspace_id: &str, id: &str) -> Result<(), String> {
        RepositoryFactory::get_ws_config_repository().delete(workspace_id, id)
    }
}
