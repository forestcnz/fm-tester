//! AI HTTP 客户端实现
//!
//! 提供 AI API 的 HTTP 请求功能，实现 AiHttpClientService trait。
//! 符合 DDD 架构规范：领域层定义接口，基础设施层实现。

use crate::domain::models::{
    AiChatMessage, ChatRequest, ChatResponse, ChatStreamResponse, Header, ModelsResponse,
    StreamResult, ToolCall, ToolCallFunction, ToolDef,
};
use crate::domain::repositories::AiHttpClientService;
use futures_util::StreamExt;
use reqwest;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

// 全局复用 AI HTTP Client（连接池 / TLS 会话复用）
// 不在 builder 上设置 timeout，由每次 RequestBuilder::timeout 单独控制。
lazy_static::lazy_static! {
    static ref AI_HTTP_CLIENT: reqwest::Client = reqwest::Client::builder()
        .danger_accept_invalid_certs(false)
        .pool_idle_timeout(Duration::from_secs(60))
        .tcp_keepalive(Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
}

/// Reqwest AI HTTP 客户端实现
pub struct ReqwestAiHttpClientService;

impl ReqwestAiHttpClientService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReqwestAiHttpClientService {
    fn default() -> Self {
        Self::new()
    }
}

impl AiHttpClientService for ReqwestAiHttpClientService {
    async fn get_models(
        &self,
        api_endpoint: &str,
        api_key: &str,
        custom_headers: Option<Vec<Header>>,
    ) -> Result<Vec<String>, String> {
        let url = format!("{}/models", api_endpoint.trim_end_matches('/'));

        // 复用全局 Client
        let client = AI_HTTP_CLIENT.clone();

        let mut request = client
            .get(&url)
            .timeout(Duration::from_secs(30))
            .header("Authorization", format!("Bearer {}", api_key));

        if let Some(headers) = custom_headers {
            for header in headers {
                if header.enabled {
                    request = request.header(&header.key, &header.value);
                }
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API returned status {}: {}", status, body));
        }

        let models: ModelsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let model_ids = models.data.iter().map(|m| m.id.clone()).collect();
        Ok(model_ids)
    }

    async fn chat(
        &self,
        api_endpoint: &str,
        api_key: &str,
        model: &str,
        messages: Vec<AiChatMessage>,
        custom_headers: Option<Vec<Header>>,
        timeout: u64,
    ) -> Result<String, String> {
        let url = format!("{}/chat/completions", api_endpoint.trim_end_matches('/'));

        let request_body = ChatRequest {
            model: model.to_string(),
            messages,
            stream: Some(false),
            tools: None,
            tool_choice: None,
        };

        let client = AI_HTTP_CLIENT.clone();
        let mut request = client
            .post(&url)
            .timeout(Duration::from_secs(timeout))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body);

        if let Some(headers) = custom_headers {
            for header in headers {
                if header.enabled {
                    request = request.header(&header.key, &header.value);
                }
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API returned status {}: {}", status, body));
        }

        let resp: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        resp.choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .filter(|c| !c.trim().is_empty())
            .ok_or_else(|| "AI 返回空内容".to_string())
    }

    async fn chat_stream(
        &self,
        app: AppHandle,
        api_endpoint: &str,
        api_key: &str,
        model: &str,
        messages: Vec<AiChatMessage>,
        custom_headers: Option<Vec<Header>>,
        timeout: u64,
        cancellation_check: Option<Box<dyn Fn() -> bool + Send + Sync>>,
    ) -> Result<String, String> {
        let url = format!("{}/chat/completions", api_endpoint.trim_end_matches('/'));

        let request_body = ChatRequest {
            model: model.to_string(),
            messages,
            stream: Some(true),
            tools: None,
            tool_choice: None,
        };

        // 复用全局 Client
        let client = AI_HTTP_CLIENT.clone();

        let mut request = client
            .post(&url)
            .timeout(Duration::from_secs(timeout))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body);

        if let Some(headers) = custom_headers {
            for header in headers {
                if header.enabled {
                    request = request.header(&header.key, &header.value);
                }
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API returned status {}: {}", status, body));
        }

        // 处理流式响应
        let mut stream = response.bytes_stream();
        let mut full_content = String::new();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            // 检查是否被取消
            if let Some(check) = &cancellation_check {
                if check() {
                    return Err("生成已取消".to_string());
                }
            }

            let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
            let chunk_text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_text);

            // 解析 SSE 格式
            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() || line == "data: [DONE]" {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(stream_resp) = serde_json::from_str::<ChatStreamResponse>(data) {
                        for choice in &stream_resp.choices {
                            if let Some(reasoning) = &choice.delta.reasoning_content {
                                app.emit("ai-chat-reasoning", reasoning).ok();
                            }
                            if let Some(content) = &choice.delta.content {
                                full_content.push_str(content);
                                app.emit("ai-chat-stream", content).ok();
                            }
                        }
                    }
                }
            }
        }

        Ok(full_content)
    }

    async fn chat_with_tools(
        &self,
        app: AppHandle,
        api_endpoint: &str,
        api_key: &str,
        model: &str,
        messages: Vec<AiChatMessage>,
        tools: Vec<ToolDef>,
        custom_headers: Option<Vec<Header>>,
        timeout: u64,
        cancellation_check: Option<Box<dyn Fn() -> bool + Send + Sync>>,
    ) -> Result<StreamResult, String> {
        let url = format!("{}/chat/completions", api_endpoint.trim_end_matches('/'));

        let request_body = ChatRequest {
            model: model.to_string(),
            messages,
            stream: Some(true),
            tools: Some(tools),
            tool_choice: Some(serde_json::json!("auto")),
        };

        let client = AI_HTTP_CLIENT.clone();
        let mut request = client
            .post(&url)
            .timeout(Duration::from_secs(timeout))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body);

        if let Some(headers) = custom_headers {
            for header in headers {
                if header.enabled {
                    request = request.header(&header.key, &header.value);
                }
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API returned status {}: {}", status, body));
        }

        let mut stream = response.bytes_stream();
        let mut result = StreamResult::default();
        let mut acc: std::collections::BTreeMap<u32, ToolCallAcc> =
            std::collections::BTreeMap::new();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            if let Some(check) = &cancellation_check {
                if check() {
                    return Err("生成已取消".to_string());
                }
            }

            let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
            let chunk_text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_text);

            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() || line == "data: [DONE]" {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(stream_resp) = serde_json::from_str::<ChatStreamResponse>(data) {
                        for choice in &stream_resp.choices {
                            if let Some(reasoning) = &choice.delta.reasoning_content {
                                app.emit("ai-chat-reasoning", reasoning).ok();
                            }
                            if let Some(content) = &choice.delta.content {
                                result.content.push_str(content);
                                app.emit("ai-chat-stream", content).ok();
                            }
                            // 累积工具调用分片（按 index 合并）
                            if let Some(tcs) = &choice.delta.tool_calls {
                                for tc in tcs {
                                    let entry = acc.entry(tc.index).or_default();
                                    if let Some(id) = &tc.id {
                                        entry.id = Some(id.clone());
                                    }
                                    if let Some(t) = &tc.tool_type {
                                        entry.tool_type = Some(t.clone());
                                    }
                                    if let Some(name) = &tc.function.name {
                                        entry.function_name = Some(name.clone());
                                    }
                                    entry.arguments.push_str(&tc.function.arguments);
                                }
                            }
                            if let Some(fr) = &choice.finish_reason {
                                result.finish_reason = Some(fr.clone());
                            }
                        }
                    }
                }
            }
        }

        // 累积器合并为完整 ToolCall
        result.tool_calls = acc
            .into_values()
            .filter_map(|a| {
                let id = a.id?;
                let name = a.function_name?;
                Some(ToolCall {
                    id,
                    tool_type: a.tool_type.unwrap_or_else(|| "function".to_string()),
                    function: ToolCallFunction {
                        name,
                        arguments: a.arguments,
                    },
                })
            })
            .collect();

        Ok(result)
    }
}

/// 流式 tool_call 分片累积器
#[derive(Default)]
struct ToolCallAcc {
    id: Option<String>,
    tool_type: Option<String>,
    function_name: Option<String>,
    arguments: String,
}

/// 全局 AI HTTP 客户端实例
pub fn get_ai_http_client() -> ReqwestAiHttpClientService {
    ReqwestAiHttpClientService::new()
}
