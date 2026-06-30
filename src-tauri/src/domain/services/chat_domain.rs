use crate::domain::models::{ChatMessage, ChatSession, ChatSessionIndex};

/// Chat 领域服务（纯业务逻辑）
pub struct ChatDomainService;

impl ChatDomainService {
    /// 从完整会话创建索引条目（纯业务逻辑）
    pub fn session_to_index_entry(session: &ChatSession) -> ChatSessionIndex {
        ChatSessionIndex {
            id: session.id.clone(),
            created_at: session.created_at.clone(),
            title: session.title.clone(),
            message_count: session.messages.len(),
        }
    }

    /// 从首条用户消息提取临时标题（取首行，截断到 30 字符）
    ///
    /// 用于在 AI 生成总结标题前提供即时可见的占位标题。
    pub fn extract_fallback_title(messages: &[ChatMessage]) -> Option<String> {
        let first_user = messages.iter().find(|m| m.role == "user")?;
        let content = first_user.content.trim();
        if content.is_empty() {
            return None;
        }
        let first_line = content.lines().next().unwrap_or(content);
        let trimmed: String = first_line.chars().take(30).collect();
        if first_line.chars().count() > 30 {
            Some(format!("{}…", trimmed))
        } else {
            Some(trimmed)
        }
    }
}
