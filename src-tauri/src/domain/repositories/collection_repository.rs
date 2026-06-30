//! 集合仓储接口
//!
//! 定义集合数据持久化的抽象接口，遵循 DDD 依赖反转原则。
//! 领域层通过此接口访问数据，具体实现在基础设施层。

use crate::domain::models::{Collection, CollectionsConfig, CollectionsIndex};

/// 集合仓储接口
///
/// 负责集合数据的持久化操作，包括：
/// - 索引文件读写（collections.toml）
/// - 单个集合项文件读写（collections/{id}.toml）
pub trait CollectionRepository {
    /// 读取集合索引
    fn read_index(&self, workspace_id: &str) -> Result<CollectionsIndex, String>;

    /// 写入集合索引
    fn write_index(&self, workspace_id: &str, index: &CollectionsIndex) -> Result<(), String>;

    /// 读取单个集合项文件
    fn read_item(&self, workspace_id: &str, id: &str) -> Result<Option<Collection>, String>;

    /// 写入单个集合项文件
    fn write_item(&self, workspace_id: &str, item: &Collection) -> Result<(), String>;

    /// 写入单个集合项并更新索引（用于单独更新操作）
    fn write_item_with_index_update(
        &self,
        workspace_id: &str,
        item: &Collection,
    ) -> Result<(), String>;

    /// 删除单个集合项文件
    fn delete_item(&self, workspace_id: &str, id: &str) -> Result<(), String>;

    /// 递归删除集合项及其所有子项文件
    fn delete_item_recursive(&self, workspace_id: &str, item: &Collection) -> Result<(), String>;

    /// 读取所有集合（从索引构建完整树）
    fn read_all(&self, workspace_id: &str) -> Result<CollectionsConfig, String>;

    /// 写入所有集合（更新索引和所有文件）
    fn write_all(&self, workspace_id: &str, config: &CollectionsConfig) -> Result<(), String>;

    /// 获取集合目录路径
    fn get_collections_dir(&self, workspace_id: &str) -> std::path::PathBuf;

    /// 获取集合索引文件路径
    fn get_index_path(&self, workspace_id: &str) -> std::path::PathBuf;

    /// 获取单个集合项文件路径
    fn get_item_path(&self, workspace_id: &str, id: &str) -> std::path::PathBuf;
}
