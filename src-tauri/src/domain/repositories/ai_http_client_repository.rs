//! AI HTTP 客户端仓储接口
//!
//! 定义 AI HTTP 客户端服务的领域接口，具体实现在基础设施层。
//! 符合 DDD 依赖反转原则，领域层依赖抽象而非具体实现。

use crate::domain::models::{AiChatMessage, Header, StreamResult, ToolDef};
use tauri::AppHandle;

/// AI HTTP 客户端服务接口（领域层接口定义）
pub trait AiHttpClientService {
    /// 获取 AI 模型列表
    fn get_models(
        &self,
        api_endpoint: &str,
        api_key: &str,
        custom_headers: Option<Vec<Header>>,
    ) -> impl std::future::Future<Output = Result<Vec<String>, String>> + Send;

    /// AI 聊天（非流式，一次性返回完整内容）
    ///
    /// 用于生成会话标题等无需流式推送的场景，不会 emit 任何事件。
    fn chat(
        &self,
        api_endpoint: &str,
        api_key: &str,
        model: &str,
        messages: Vec<AiChatMessage>,
        custom_headers: Option<Vec<Header>>,
        timeout: u64,
    ) -> impl std::future::Future<Output = Result<String, String>> + Send;

    /// AI 聊天（流式响应）
    fn chat_stream(
        &self,
        app: AppHandle,
        api_endpoint: &str,
        api_key: &str,
        model: &str,
        messages: Vec<AiChatMessage>,
        custom_headers: Option<Vec<Header>>,
        timeout: u64,
        cancellation_check: Option<Box<dyn Fn() -> bool + Send + Sync>>,
    ) -> impl std::future::Future<Output = Result<String, String>> + Send;

    /// AI 聊天（带工具调用，流式）
    ///
    /// 返回累积的文本内容与完整的工具调用列表；
    /// content / reasoning 仍通过 `ai-chat-stream` / `ai-chat-reasoning` 事件流式推送。
    fn chat_with_tools(
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
    ) -> impl std::future::Future<Output = Result<StreamResult, String>> + Send;
}
