//! 导入应用服务
//!
//! 处理导入/导出相关的 UI 交互，协调 CollectionApplicationService 和 ImportDomainService。

use crate::application::services::{read_collections, write_collections};
use crate::domain::models::common::generate_id;
use crate::domain::models::import::ParsedCurl;
use crate::domain::models::Collection;
use crate::domain::services::{
    convert_collection_to_postman, convert_postman_to_collection, convert_to_collection,
    find_collection_item, find_item_in_collections, parse_curl_command, parse_openapi,
    parse_postman,
};

/// 导入应用服务
pub struct ImportApplicationService;

impl ImportApplicationService {
    /// 创建默认实例
    pub fn new() -> Self {
        Self
    }

    /// 为集合分配新 ID
    fn assign_new_ids(collection: &mut Collection) {
        collection.id = generate_id("col");
        for child in &mut collection.children {
            Self::assign_new_ids(child);
        }
    }

    /// 预览 OpenAPI 导入
    pub fn preview_openapi(
        content: &str,
        format: &str,
        root_name: Option<String>,
    ) -> Result<Collection, String> {
        let openapi = parse_openapi(content, format)?;
        let collection = convert_to_collection(openapi, None, root_name)?;
        Ok(collection)
    }

    /// 导入 OpenAPI
    pub fn import_openapi(
        workspace_id: &str,
        content: &str,
        format: &str,
        target_collection_id: Option<&str>,
        root_name: Option<String>,
    ) -> Result<Collection, String> {
        let openapi = parse_openapi(content, format)?;

        let mut root_collection = convert_to_collection(openapi, None, root_name)?;

        let mut config = read_collections(workspace_id)?;

        if let Some(parent_id) = target_collection_id {
            let parent = find_collection_item(&mut config.collections, parent_id)
                .ok_or_else(|| format!("目标集合不存在: {}", parent_id))?;

            if parent.item_type != "collection" {
                return Err("目标必须是集合".to_string());
            }

            for child in &mut root_collection.children {
                Self::assign_new_ids(child);
                parent.children.push(child.clone());
            }
            write_collections(workspace_id, &config)?;
            Ok(root_collection)
        } else {
            Self::assign_new_ids(&mut root_collection);
            config.collections.push(root_collection.clone());
            write_collections(workspace_id, &config)?;
            Ok(root_collection)
        }
    }

    /// 解析 curl 命令
    pub fn parse_curl(curl_command: &str) -> Result<ParsedCurl, String> {
        parse_curl_command(curl_command)
    }

    /// 预览 Postman 导入
    pub fn preview_postman(content: &str, root_name: Option<String>) -> Result<Collection, String> {
        let postman = parse_postman(content)?;
        let collection = convert_postman_to_collection(postman, root_name)?;
        Ok(collection)
    }

    /// 导入 Postman
    pub fn import_postman(
        workspace_id: &str,
        content: &str,
        target_collection_id: Option<&str>,
        root_name: Option<String>,
    ) -> Result<Collection, String> {
        let postman = parse_postman(content)?;
        let mut root_collection = convert_postman_to_collection(postman, root_name)?;

        let mut config = read_collections(workspace_id)?;

        if let Some(parent_id) = target_collection_id {
            let parent = find_collection_item(&mut config.collections, parent_id)
                .ok_or_else(|| format!("目标集合不存在: {}", parent_id))?;

            if parent.item_type != "collection" {
                return Err("目标必须是集合".to_string());
            }

            Self::assign_new_ids(&mut root_collection);
            for child in &root_collection.children {
                parent.children.push(child.clone());
            }
            write_collections(workspace_id, &config)?;
            Ok(root_collection)
        } else {
            Self::assign_new_ids(&mut root_collection);
            config.collections.push(root_collection.clone());
            write_collections(workspace_id, &config)?;
            Ok(root_collection)
        }
    }

    /// 导出集合为 Postman 格式（从文件读取）
    pub fn export_collection_postman(
        workspace_id: &str,
        collection_id: &str,
    ) -> Result<String, String> {
        let config = read_collections(workspace_id)?;

        let collection = find_item_in_collections(&config.collections, collection_id)
            .ok_or_else(|| format!("集合不存在: {}", collection_id))?;

        if collection.item_type != "collection" {
            return Err("只能导出集合".to_string());
        }

        let postman_collection = convert_collection_to_postman(collection);

        serde_json::to_string_pretty(&postman_collection).map_err(|e| format!("序列化失败: {}", e))
    }

    /// 导出集合为 Postman 格式（使用前端处理过的数据）
    pub fn export_collection_postman_with_data(collection: &Collection) -> Result<String, String> {
        if collection.item_type != "collection" {
            return Err("只能导出集合".to_string());
        }

        let postman_collection = convert_collection_to_postman(collection);

        serde_json::to_string_pretty(&postman_collection).map_err(|e| format!("序列化失败: {}", e))
    }
}

impl Default for ImportApplicationService {
    fn default() -> Self {
        Self::new()
    }
}
