use crate::domain::models::{ChatIndex, ChatSession};
use std::path::PathBuf;

/// Chat 仓库接口
pub trait ChatRepository {
    /// 获取 chat 目录路径
    fn get_chat_dir(&self, workspace_id: &str) -> PathBuf;

    /// 获取索引文件路径
    fn get_index_path(&self, workspace_id: &str) -> PathBuf;

    /// 获取会话文件路径
    fn get_session_path(&self, workspace_id: &str, session_id: &str) -> PathBuf;

    /// 确保 chat 目录存在
    fn ensure_dir(&self, workspace_id: &str) -> Result<(), String>;

    /// 读取聊天索引
    fn read_index(&self, workspace_id: &str) -> Result<ChatIndex, String>;

    /// 写入聊天索引
    fn write_index(&self, workspace_id: &str, index: &ChatIndex) -> Result<(), String>;

    /// 读取单个会话
    fn read_session(
        &self,
        workspace_id: &str,
        session_id: &str,
    ) -> Result<Option<ChatSession>, String>;

    /// 写入单个会话
    fn write_session(&self, workspace_id: &str, session: &ChatSession) -> Result<(), String>;

    /// 删除会话文件
    fn delete_session_file(&self, workspace_id: &str, session_id: &str) -> Result<(), String>;
}
