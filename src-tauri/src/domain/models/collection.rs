use super::{SavedResponseIndexEntry, Variable, WsConfig};
use serde::{Deserialize, Serialize};

/// HTTP 请求头（值对象）
///
/// 不可变的值对象，用于表示单个 HTTP 请求头。
/// 通过 `enabled` 字段控制是否在请求中包含该请求头。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Form 表单字段（值对象）
///
/// 用于表示 form-data 或 x-www-form-urlencoded 请求中的字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub key: String,
    pub value: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileInfo>>,
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
}

/// 集合（聚合根 Aggregate Root）
///
/// 这是集合聚合的根实体，管理所有子集合和 API 接口。
///
/// ## 聚合边界
/// - 一个 Collection 聚合包含：集合本身 + 所有子集合 + 所有 API 接口
/// - 子集合是独立的聚合根，但层级关系通过索引维护
/// - API 接口是聚合的一部分，不能独立存在
///
/// ## 业务规则
/// - 集合层级最多三层（MAX_DEPTH = 2）
/// - 集合名称不能为空
/// - 集合类型必须是 "collection"、"api" 或 "websocket"
///
/// ## 生命周期
/// - 创建：通过 `CollectionDomainService::create_collection_entity()`
/// - 验证：通过 `Collection.validate()` 方法
/// - 持久化：通过 `CollectionRepository` 仓储接口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub item_type: String,
    pub children: Vec<Collection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<Param>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<Header>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form_fields: Option<Vec<FormField>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saved_responses: Option<Vec<SavedResponseIndexEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub common_headers: Option<Vec<Header>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_variables: Option<Vec<Variable>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ws_config: Option<WsConfig>,
}

impl Collection {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("集合名称不能为空".to_string());
        }
        if !["collection", "api", "websocket"].contains(&self.item_type.as_str()) {
            return Err("项类型必须是 collection、api 或 websocket".to_string());
        }
        if self.item_type == "api" {
            if let Some(ref method) = self.method {
                if !["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"]
                    .contains(&method.to_uppercase().as_str())
                {
                    return Err(format!("不支持的 HTTP 方法: {}", method));
                }
            }
        }
        if self.item_type == "websocket" {
            if let Some(ref ws_config) = self.ws_config {
                ws_config.validate()?;
            } else {
                return Err("WebSocket 配置不能为空".to_string());
            }
        }
        Ok(())
    }

    pub fn is_collection(&self) -> bool {
        self.item_type == "collection"
    }

    pub fn is_api(&self) -> bool {
        self.item_type == "api"
    }

    pub fn is_websocket(&self) -> bool {
        self.item_type == "websocket"
    }

    pub fn get_depth(&self, current_depth: usize) -> usize {
        if self.children.is_empty() {
            return current_depth;
        }
        let mut max_depth = current_depth;
        for child in &self.children {
            let child_depth = child.get_depth(current_depth + 1);
            if child_depth > max_depth {
                max_depth = child_depth;
            }
        }
        max_depth
    }
}

/// Query 参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 合配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectionsConfig {
    pub collections: Vec<Collection>,
}

/// 集合索引项（用于 collections.toml）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionIndexItem {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(default)]
    pub children: Vec<CollectionIndexItem>,
}

/// 集合索引结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectionsIndex {
    #[serde(default)]
    pub items: Vec<CollectionIndexItem>,
}

/// 带祖先链的项（用于轻量级加载）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemWithAncestors {
    pub item: Collection,
    pub ancestors: Vec<Collection>,
}

impl CollectionsConfig {
    pub fn validate(&self) -> Result<(), String> {
        for collection in &self.collections {
            collection.validate()?;
            self.validate_collection_depth(collection, 0)?;
        }
        Ok(())
    }

    fn validate_collection_depth(
        &self,
        collection: &Collection,
        current_depth: usize,
    ) -> Result<(), String> {
        if current_depth > 2 {
            return Err(format!("集合层级超过限制（最多三层）: {}", collection.name));
        }
        for child in &collection.children {
            self.validate_collection_depth(child, current_depth + 1)?;
        }
        Ok(())
    }

    pub fn find_collection_by_id(&self, id: &str) -> Option<&Collection> {
        for collection in &self.collections {
            if collection.id == id {
                return Some(collection);
            }
            if let Some(found) = self.find_collection_in_children(&collection.children, id) {
                return Some(found);
            }
        }
        None
    }

    fn find_collection_in_children<'a>(
        &self,
        children: &'a [Collection],
        id: &str,
    ) -> Option<&'a Collection> {
        for child in children {
            if child.id == id {
                return Some(child);
            }
            if let Some(found) = self.find_collection_in_children(&child.children, id) {
                return Some(found);
            }
        }
        None
    }
}
