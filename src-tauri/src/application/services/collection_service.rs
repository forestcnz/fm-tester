//! 集合应用服务
//!
//! 处理集合相关的业务逻辑，通过仓储工厂动态获取仓储。

use crate::domain::models::{Collection, CollectionIndexItem, CollectionsConfig, CollectionsIndex};
use crate::domain::services::{
    find_ancestor_chain, find_api_in_collections, find_collection_item, find_item_in_collections,
    find_parent_children, get_all_descendant_ids, get_collection_depth,
    get_collection_max_child_depth, remove_collection_item, CollectionDomainService,
};
use crate::infrastructure::RepositoryFactory;

/// 集合应用服务
///
/// 不持有仓储实例，每次调用时通过工厂根据 workspace_id 获取对应仓储
pub struct CollectionApplicationService;

impl CollectionApplicationService {
    /// 创建集合（纯验证，不持久化）
    pub fn create_collection(
        name: String,
        description: Option<String>,
    ) -> Result<Collection, String> {
        CollectionDomainService::validate_collection_name(&name)?;
        let collection = CollectionDomainService::create_collection_entity(name, description);
        CollectionDomainService::validate_collection_item(&collection)?;
        Ok(collection)
    }

    /// 验证集合名称
    pub fn validate_collection_name(name: &str) -> Result<(), String> {
        CollectionDomainService::validate_collection_name(name)
    }

    /// 生成集合 ID
    pub fn generate_collection_id() -> String {
        CollectionDomainService::generate_collection_id()
    }

    /// 生成 API ID
    pub fn generate_api_id() -> String {
        CollectionDomainService::generate_api_id()
    }

    /// 查找 API
    pub fn find_api(config: &CollectionsConfig, api_id: &str) -> Option<Collection> {
        find_api_in_collections(&config.collections, api_id).cloned()
    }

    /// 查找集合项（可变版本）
    pub fn find_item_mut<'a>(
        config: &'a mut CollectionsConfig,
        item_id: &str,
    ) -> Option<&'a mut Collection> {
        find_collection_item(&mut config.collections, item_id)
    }

    /// 查找集合项（不可变版本）
    pub fn find_item(config: &CollectionsConfig, item_id: &str) -> Option<Collection> {
        find_item_in_collections(&config.collections, item_id).cloned()
    }

    /// 获取父集合的 children 数组
    pub fn find_parent_children_mut<'a>(
        config: &'a mut CollectionsConfig,
        parent_id: Option<&str>,
    ) -> Option<&'a mut Vec<Collection>> {
        find_parent_children(&mut config.collections, parent_id)
    }

    /// 获取集合深度
    pub fn get_depth(config: &CollectionsConfig, collection_id: &str) -> Option<usize> {
        get_collection_depth(&config.collections, collection_id, 0)
    }

    /// 获取集合的所有子孙 ID
    pub fn get_descendant_ids(
        config: &CollectionsConfig,
        collection_id: &str,
    ) -> Option<Vec<String>> {
        get_all_descendant_ids(&config.collections, collection_id)
    }

    /// 获取集合的最大子层级深度
    pub fn get_max_child_depth(config: &CollectionsConfig, collection_id: &str) -> Option<usize> {
        get_collection_max_child_depth(&config.collections, collection_id)
    }

    /// 删除集合项
    pub fn remove_item(config: &mut CollectionsConfig, item_id: &str) -> bool {
        remove_collection_item(&mut config.collections, item_id)
    }

    /// 查找祖先链
    pub fn find_ancestor_chain(
        config: &CollectionsConfig,
        target_id: &str,
        path: &mut Vec<Collection>,
    ) -> bool {
        find_ancestor_chain(&config.collections, target_id, path)
    }

    /// 从集合构建索引项
    pub fn build_index_from_collection(collection: &Collection) -> CollectionIndexItem {
        CollectionIndexItem {
            id: collection.id.clone(),
            name: collection.name.clone(),
            item_type: collection.item_type.clone(),
            method: collection.method.clone(),
            children: collection
                .children
                .iter()
                .map(Self::build_index_from_collection)
                .collect(),
        }
    }

    // ==================== 实例方法（动态获取仓储）====================

    /// 读取所有集合
    pub fn read_all(workspace_id: &str) -> Result<CollectionsConfig, String> {
        let repository = RepositoryFactory::get_collection_repository();
        repository.read_all(workspace_id)
    }

    /// 写入所有集合
    pub fn write_all(workspace_id: &str, config: &CollectionsConfig) -> Result<(), String> {
        let repository = RepositoryFactory::get_collection_repository();
        repository.write_all(workspace_id, config)
    }

    /// 写入集合索引
    pub fn write_index(workspace_id: &str, index: &CollectionsIndex) -> Result<(), String> {
        let repository = RepositoryFactory::get_collection_repository();
        repository.write_index(workspace_id, index)
    }

    /// 写入单个集合项（不包含 children）
    pub fn write_item(workspace_id: &str, item: &Collection) -> Result<(), String> {
        let repository = RepositoryFactory::get_collection_repository();

        let item_for_file = Collection {
            id: item.id.clone(),
            name: item.name.clone(),
            description: item.description.clone(),
            item_type: item.item_type.clone(),
            children: Vec::new(),
            method: item.method.clone(),
            url: item.url.clone(),
            params: item.params.clone(),
            headers: item.headers.clone(),
            body: item.body.clone(),
            body_type: item.body_type.clone(),
            form_fields: item.form_fields.clone(),
            saved_responses: item.saved_responses.clone(),
            common_headers: item.common_headers.clone(),
            collection_variables: item.collection_variables.clone(),
            ws_config: item.ws_config.clone(),
        };

        repository.write_item(workspace_id, &item_for_file)
    }

    /// 写入单个集合项并更新索引（用于单独更新操作）
    pub fn write_item_with_index_update(
        workspace_id: &str,
        item: &Collection,
    ) -> Result<(), String> {
        let repository = RepositoryFactory::get_collection_repository();

        let item_for_file = Collection {
            id: item.id.clone(),
            name: item.name.clone(),
            description: item.description.clone(),
            item_type: item.item_type.clone(),
            children: Vec::new(),
            method: item.method.clone(),
            url: item.url.clone(),
            params: item.params.clone(),
            headers: item.headers.clone(),
            body: item.body.clone(),
            body_type: item.body_type.clone(),
            form_fields: item.form_fields.clone(),
            saved_responses: item.saved_responses.clone(),
            common_headers: item.common_headers.clone(),
            collection_variables: item.collection_variables.clone(),
            ws_config: item.ws_config.clone(),
        };

        repository.write_item_with_index_update(workspace_id, &item_for_file)
    }

    /// 递归删除集合项文件
    pub fn delete_item_recursive(workspace_id: &str, item: &Collection) -> Result<(), String> {
        let repository = RepositoryFactory::get_collection_repository();
        repository.delete_item_recursive(workspace_id, item)
    }
}

// ==================== 模块级公共函数（向后兼容）====================

/// 读取所有集合
pub fn read_collections(workspace_id: &str) -> Result<CollectionsConfig, String> {
    CollectionApplicationService::read_all(workspace_id)
}

/// 写入所有集合
pub fn write_collections(workspace_id: &str, config: &CollectionsConfig) -> Result<(), String> {
    CollectionApplicationService::write_all(workspace_id, config)
}

/// 写入集合索引
pub fn write_collections_index(workspace_id: &str, index: &CollectionsIndex) -> Result<(), String> {
    CollectionApplicationService::write_index(workspace_id, index)
}

/// 写入单个集合项
pub fn write_single_item(workspace_id: &str, item: &Collection) -> Result<(), String> {
    CollectionApplicationService::write_item(workspace_id, item)
}

/// 写入单个集合项并更新索引（用于单独更新操作）
pub fn write_single_item_with_index_update(
    workspace_id: &str,
    item: &Collection,
) -> Result<(), String> {
    CollectionApplicationService::write_item_with_index_update(workspace_id, item)
}

/// 递归删除集合项文件
pub fn delete_item_files_recursive(workspace_id: &str, item: &Collection) -> Result<(), String> {
    CollectionApplicationService::delete_item_recursive(workspace_id, item)
}

/// 从集合构建索引项
pub fn build_index_from_collection(collection: &Collection) -> CollectionIndexItem {
    CollectionApplicationService::build_index_from_collection(collection)
}
