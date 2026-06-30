//! SQLite Chat 仓储实现
//!
//! 消息存储使用独立的 chat_messages 表，支持分页查询。

use crate::domain::models::{ChatIndex, ChatMessage, ChatSession, ChatSessionIndex};
use crate::domain::repositories::ChatRepository;
use crate::infrastructure::sqlite::connection::{with_connection, with_transaction};
use crate::repo_error;
use rusqlite::params;

pub struct SqliteChatRepository;

impl SqliteChatRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SqliteChatRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatRepository for SqliteChatRepository {
    fn get_chat_dir(&self, workspace_id: &str) -> std::path::PathBuf {
        crate::infrastructure::data_dir::get_workspace_dir(workspace_id)
    }

    fn get_index_path(&self, workspace_id: &str) -> std::path::PathBuf {
        crate::infrastructure::data_dir::get_workspace_dir(workspace_id)
    }

    fn get_session_path(&self, workspace_id: &str, _session_id: &str) -> std::path::PathBuf {
        crate::infrastructure::data_dir::get_workspace_dir(workspace_id)
    }

    fn ensure_dir(&self, workspace_id: &str) -> Result<(), String> {
        let dir = crate::infrastructure::data_dir::get_workspace_dir(workspace_id);
        std::fs::create_dir_all(&dir).map_err(|e| repo_error!("创建工作区目录失败: {}", e))?;
        Ok(())
    }

    fn read_index(&self, workspace_id: &str) -> Result<ChatIndex, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare("SELECT id, created_at, title, active_session FROM chat_sessions ORDER BY created_at")
                .map_err(|e| repo_error!("准备查询聊天索引失败: {}", e))?;

            let rows: Vec<(String, String, Option<String>, i32)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, i32>(3)?,
                    ))
                })
                .map_err(|e| repo_error!("查询聊天索引失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析聊天索引行数据失败: {}", e))?;

            let mut sessions = Vec::new();
            let mut active_session_id: Option<String> = None;

            for (id, created_at, title, active_flag) in rows {
                if active_flag != 0 {
                    active_session_id = Some(id.clone());
                }

                let message_count: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM chat_messages WHERE session_id = ?1",
                        params![&id],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                sessions.push(ChatSessionIndex {
                    id,
                    created_at,
                    title,
                    message_count: message_count as usize,
                });
            }

            Ok(ChatIndex {
                sessions,
                active_session_id,
            })
        })
    }

    fn write_index(&self, workspace_id: &str, index: &ChatIndex) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_transaction(&ws, |conn| {
            conn.execute("UPDATE chat_sessions SET active_session = 0", [])
                .map_err(|e| repo_error!("重置激活会话标志失败: {}", e))?;

            if let Some(ref active_id) = index.active_session_id {
                conn.execute(
                    "UPDATE chat_sessions SET active_session = 1 WHERE id = ?1",
                    params![active_id],
                )
                .map_err(|e| repo_error!("设置激活会话标志失败: {}", e))?;
            }

            for session in &index.sessions {
                conn.execute(
                    "UPDATE chat_sessions SET title = ?1 WHERE id = ?2",
                    params![session.title, session.id],
                )
                .map_err(|e| repo_error!("更新会话元数据失败: {}", e))?;
            }

            Ok(())
        })
    }

    fn read_session(
        &self,
        workspace_id: &str,
        session_id: &str,
    ) -> Result<Option<ChatSession>, String> {
        let ws = workspace_id.to_string();
        let sid = session_id.to_string();
        with_connection(&ws, |conn| {
            let result = conn.query_row(
                "SELECT id, created_at, title FROM chat_sessions WHERE id = ?1",
                params![&sid],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                },
            );

            match result {
                Ok((id, created_at, title)) => {
                    let mut stmt = conn
                        .prepare(
                            "SELECT role, content, reasoning, timestamp \
                             FROM chat_messages WHERE session_id = ?1 ORDER BY rowid",
                        )
                        .map_err(|e| repo_error!("准备查询聊天消息失败: {}", e))?;

                    let messages: Vec<ChatMessage> = stmt
                        .query_map(params![&sid], |row| {
                            Ok(ChatMessage {
                                role: row.get::<_, String>(0)?,
                                content: row.get::<_, String>(1)?,
                                reasoning: row.get::<_, Option<String>>(2)?,
                                timestamp: row.get::<_, Option<String>>(3)?,
                            })
                        })
                        .map_err(|e| repo_error!("查询聊天消息失败: {}", e))?
                        .collect::<Result<Vec<_>, rusqlite::Error>>()
                        .map_err(|e| repo_error!("解析聊天消息行数据失败: {}", e))?;

                    Ok(Some(ChatSession {
                        id,
                        created_at,
                        messages,
                        title,
                    }))
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(repo_error!("查询聊天会话失败: {}", e)),
            }
        })
    }

    fn write_session(&self, workspace_id: &str, session: &ChatSession) -> Result<(), String> {
        let ws = workspace_id.to_string();
        let session_id = session.id.clone();
        let created_at = session.created_at.clone();
        let title = session.title.clone();
        let messages = session.messages.clone();

        with_transaction(&ws, |conn| {
            conn.execute(
                "INSERT OR REPLACE INTO chat_sessions (id, created_at, title, active_session) \
                 VALUES (?1, ?2, ?3, 0)",
                params![&session_id, &created_at, &title],
            )
            .map_err(|e| repo_error!("写入会话失败: {}", e))?;

            conn.execute(
                "DELETE FROM chat_messages WHERE session_id = ?1",
                params![&session_id],
            )
            .map_err(|e| repo_error!("清空旧消息失败: {}", e))?;

            for msg in &messages {
                conn.execute(
                    "INSERT INTO chat_messages (id, session_id, role, content, reasoning, timestamp) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        crate::domain::models::generate_id("chat_msg"),
                        &session_id,
                        msg.role,
                        msg.content,
                        msg.reasoning,
                        msg.timestamp,
                    ],
                )
                .map_err(|e| repo_error!("写入消息失败: {}", e))?;
            }

            Ok(())
        })
    }

    fn delete_session_file(&self, workspace_id: &str, session_id: &str) -> Result<(), String> {
        let ws = workspace_id.to_string();
        let sid = session_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute("DELETE FROM chat_sessions WHERE id = ?1", params![&sid])
                .map_err(|e| repo_error!("删除会话失败: {}", e))?;

            Ok(())
        })
    }
}
