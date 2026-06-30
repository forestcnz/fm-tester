//! Chat 应用服务
//!
//! 处理聊天相关的 UI 交互，通过仓储工厂动态获取仓储。

use crate::application::services::AiApplicationService;
use crate::domain::models::common::generate_id;
use crate::domain::models::{AiChatMessage, ChatMessage, ChatSession};
use crate::domain::services::ChatDomainService;
use crate::infrastructure::RepositoryFactory;
use tauri::{AppHandle, Emitter};

/// Chat 应用服务
///
/// 无状态服务，每次调用时通过工厂根据 workspace_id 获取对应仓储
pub struct ChatApplicationService;

impl ChatApplicationService {
    /// 创建默认实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for ChatApplicationService {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatApplicationService {
    /// 保存聊天记录
    pub fn save_chat_history(
        &self,
        app: &AppHandle,
        workspace_id: &str,
        session_id: Option<String>,
        messages: Vec<ChatMessage>,
    ) -> Result<String, String> {
        if workspace_id.is_empty() {
            return Err("Workspace path is empty".to_string());
        }

        let repo = RepositoryFactory::get_chat_repository();

        // 生成 session_id（如果没有提供）
        let session_id = session_id.unwrap_or_else(|| generate_id("chat"));

        // 读取现有索引
        let mut index = repo.read_index(workspace_id)?;

        // 获取现有标题（如果是更新已有会话）
        let existing_title = index
            .sessions
            .iter()
            .find(|s| s.id == session_id)
            .and_then(|s| s.title.clone());

        // 尚无标题：先用首条用户消息生成临时标题（立即可见），并标记需后台生成 AI 总结标题
        let needs_ai_title = existing_title.is_none();
        let title = existing_title.or_else(|| ChatDomainService::extract_fallback_title(&messages));

        // 创建或更新会话
        let session = ChatSession {
            id: session_id.clone(),
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            messages: messages.clone(),
            title,
        };

        // 写入会话文件
        repo.write_session(workspace_id, &session)?;

        // 更新索引（使用 Domain Service）
        let index_entry = ChatDomainService::session_to_index_entry(&session);
        if let Some(pos) = index.sessions.iter().position(|s| s.id == session_id) {
            index.sessions[pos] = index_entry;
        } else {
            index.sessions.insert(0, index_entry);
        }
        index.active_session_id = Some(session_id.clone());

        // 写入索引文件
        repo.write_index(workspace_id, &index)?;

        // 发送事件通知前端刷新会话列表
        app.emit("chat-session-saved", &session_id).ok();

        // 后台异步生成 AI 总结标题（仅当会话此前无标题时，避免重复触发）
        if needs_ai_title {
            let ai_messages: Vec<AiChatMessage> = messages
                .iter()
                .filter(|m| m.role == "user" || m.role == "assistant")
                .take(6)
                .map(|m| AiChatMessage {
                    role: m.role.clone(),
                    content: Some(m.content.clone()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                })
                .collect();

            if !ai_messages.is_empty() {
                let app_handle = app.clone();
                let workspace_id = workspace_id.to_string();
                let session_id = session_id.clone();
                tauri::async_runtime::spawn(async move {
                    if let Ok(new_title) =
                        AiApplicationService::generate_chat_title(ai_messages).await
                    {
                        let service = ChatApplicationService;
                        if service
                            .rename_chat_session(&workspace_id, &session_id, &new_title)
                            .is_ok()
                        {
                            app_handle
                                .emit("chat-session-title-updated", &session_id)
                                .ok();
                        }
                    }
                });
            }
        }

        Ok(session_id)
    }

    /// 获取聊天记录
    pub fn get_chat_history(
        &self,
        workspace_id: &str,
        session_id: Option<String>,
    ) -> Result<Vec<ChatMessage>, String> {
        if workspace_id.is_empty() {
            return Err("Workspace path is empty".to_string());
        }

        let repo = RepositoryFactory::get_chat_repository();
        let index = repo.read_index(workspace_id)?;

        // 获取指定会话或活动会话
        let target_id = session_id.or(index.active_session_id);

        if let Some(id) = target_id {
            let session_opt = repo.read_session(workspace_id, &id)?;
            if let Some(session) = session_opt {
                return Ok(session.messages);
            }
        }

        // 返回空列表
        Ok(Vec::new())
    }

    /// 清空聊天记录
    pub fn clear_chat_history(
        &self,
        workspace_id: &str,
        session_id: Option<String>,
    ) -> Result<(), String> {
        if workspace_id.is_empty() {
            return Err("Workspace path is empty".to_string());
        }

        let repo = RepositoryFactory::get_chat_repository();
        let mut index = repo.read_index(workspace_id)?;

        // 清空指定会话或活动会话
        if let Some(id) = session_id.or(index.active_session_id.clone()) {
            // 删除会话文件
            repo.delete_session_file(workspace_id, &id)?;

            // 更新索引
            index.sessions.retain(|s| s.id != id);
            if index.active_session_id == Some(id) {
                index.active_session_id = None;
            }

            repo.write_index(workspace_id, &index)?;
        }

        Ok(())
    }

    /// 获取聊天会话列表（仅索引信息）
    pub fn get_chat_sessions(&self, workspace_id: &str) -> Result<Vec<ChatSession>, String> {
        if workspace_id.is_empty() {
            return Err("Workspace path is empty".to_string());
        }

        let repo = RepositoryFactory::get_chat_repository();
        let index = repo.read_index(workspace_id)?;

        // 将索引条目转换为会话对象（不含消息内容，节省内存）
        let sessions: Vec<ChatSession> = index
            .sessions
            .into_iter()
            .map(|s| ChatSession {
                id: s.id,
                created_at: s.created_at,
                messages: Vec::new(), // 列表不需要消息内容
                title: s.title,
            })
            .collect();

        Ok(sessions)
    }

    /// 删除聊天会话
    pub fn delete_chat_session(&self, workspace_id: &str, session_id: &str) -> Result<(), String> {
        if workspace_id.is_empty() {
            return Err("Workspace path is empty".to_string());
        }

        let repo = RepositoryFactory::get_chat_repository();

        // 删除会话文件
        repo.delete_session_file(workspace_id, session_id)?;

        // 更新索引
        let mut index = repo.read_index(workspace_id)?;
        index.sessions.retain(|s| s.id != session_id);

        if index.active_session_id == Some(session_id.to_string()) {
            index.active_session_id = None;
        }

        repo.write_index(workspace_id, &index)?;

        Ok(())
    }

    /// 重命名聊天会话
    pub fn rename_chat_session(
        &self,
        workspace_id: &str,
        session_id: &str,
        new_title: &str,
    ) -> Result<(), String> {
        if workspace_id.is_empty() {
            return Err("Workspace path is empty".to_string());
        }

        let repo = RepositoryFactory::get_chat_repository();

        // 读取并更新会话文件
        let session_opt = repo.read_session(workspace_id, session_id)?;
        if let Some(mut session) = session_opt {
            session.title = Some(new_title.to_string());
            repo.write_session(workspace_id, &session)?;

            // 更新索引
            let mut index = repo.read_index(workspace_id)?;
            if let Some(entry) = index.sessions.iter_mut().find(|s| s.id == session_id) {
                entry.title = Some(new_title.to_string());
                repo.write_index(workspace_id, &index)?;
            }

            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }
}
