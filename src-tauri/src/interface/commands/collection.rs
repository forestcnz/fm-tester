use crate::application::services::{
    build_index_from_collection, delete_item_files_recursive, read_collections,
    write_collections_index, write_single_item, write_single_item_with_index_update,
    CollectionApplicationService, ResponseApplicationService, ScriptApplicationService,
};
use crate::domain::models::{
    Collection, CollectionsConfig, FormField, Header, Param, ScriptIndexEntry, ScriptTargetType,
    Variable,
};
use crate::domain::models::{CollectionIndexItem, CollectionsIndex, ItemWithAncestors};
use crate::infrastructure::RepositoryFactory;
use std::collections::HashMap;

fn load_saved_responses_for_apis(
    collections: &mut [Collection],
    workspace_id: &str,
) -> Result<(), String> {
    for item in collections.iter_mut() {
        if item.item_type == "api" {
            let responses = ResponseApplicationService::get_by_api(workspace_id, &item.id)?;
            if !responses.is_empty() {
                item.saved_responses = Some(responses);
            }
        }
        load_saved_responses_for_apis(&mut item.children, workspace_id)?;
    }
    Ok(())
}

/// 从索引树中查找目标项的祖先 ID 链（从根到父，不包括目标自身）
fn find_ancestor_ids_in_index(
    items: &[CollectionIndexItem],
    target_id: &str,
    current_path: &[String],
) -> Option<Vec<String>> {
    for item in items {
        if item.id == target_id {
            return Some(current_path.to_vec());
        }
        let new_path: Vec<String> = current_path
            .iter()
            .chain(std::iter::once(&item.id))
            .cloned()
            .collect();
        if let Some(found) = find_ancestor_ids_in_index(&item.children, target_id, &new_path) {
            return Some(found);
        }
    }
    None
}

fn update_index_only(workspace_id: &str, config: &CollectionsConfig) -> Result<(), String> {
    let index = CollectionsIndex {
        items: config
            .collections
            .iter()
            .map(build_index_from_collection)
            .collect(),
    };
    write_collections_index(workspace_id, &index)?;
    Ok(())
}

/// 根据 ID 列表获取集合项（轻量级，不加载保存响应）
/// 用于启动时只加载打开的标签页数据
/// 返回每个目标项及其祖先链（从根到父）
#[tauri::command]
pub fn get_items_by_ids(
    workspace_id: String,
    item_ids: Vec<String>,
) -> Result<Vec<ItemWithAncestors>, String> {
    if item_ids.is_empty() {
        return Ok(Vec::new());
    }

    let repo = RepositoryFactory::get_collection_repository();

    let index = repo.read_index(&workspace_id)?;

    // 2. 收集所有需要读取的 ID（目标 + 祖先）
    let mut all_needed_ids: Vec<String> = Vec::new();
    let mut ancestor_ids_map: HashMap<String, Vec<String>> = HashMap::new();

    for target_id in &item_ids {
        if let Some(ancestor_ids) = find_ancestor_ids_in_index(&index.items, target_id, &[]) {
            ancestor_ids_map.insert(target_id.clone(), ancestor_ids.clone());
            for ancestor_id in &ancestor_ids {
                if !all_needed_ids.contains(ancestor_id) {
                    all_needed_ids.push(ancestor_id.clone());
                }
            }
        }
        if !all_needed_ids.contains(target_id) {
            all_needed_ids.push(target_id.clone());
        }
    }

    // 3. 批量读取需要的项文件
    let mut items_cache: HashMap<String, Collection> = HashMap::new();
    for id in &all_needed_ids {
        if let Some(item) = repo.read_item(&workspace_id, id)? {
            items_cache.insert(id.clone(), item);
        }
    }

    // 4. 构建返回结果
    let mut results: Vec<ItemWithAncestors> = Vec::new();
    for target_id in &item_ids {
        if let Some(item) = items_cache.get(target_id).cloned() {
            let ancestor_ids = ancestor_ids_map.get(target_id).cloned().unwrap_or_default();
            let ancestors: Vec<Collection> = ancestor_ids
                .iter()
                .filter_map(|id| items_cache.get(id).cloned())
                .collect();
            results.push(ItemWithAncestors { item, ancestors });
        }
    }

    Ok(results)
}

/// 获取集合列表
#[tauri::command]
pub fn get_collections(workspace_id: String) -> Result<Vec<Collection>, String> {
    let mut collections = read_collections(&workspace_id)?.collections;
    load_saved_responses_for_apis(&mut collections, &workspace_id)?;
    Ok(collections)
}

/// 获取集合树结构（轻量级，仅返回索引数据）
/// 注意：不再预加载保存响应，改为前端按需加载（toggleResponses）
#[tauri::command]
pub fn get_collections_tree(workspace_id: String) -> Result<Vec<CollectionIndexItem>, String> {
    let repository = RepositoryFactory::get_collection_repository();
    let index = repository.read_index(&workspace_id)?;
    Ok(index.items)
}

#[tauri::command]
pub fn create_collection(
    workspace_id: String,
    name: String,
    description: Option<String>,
    parent_id: Option<String>,
) -> Result<Collection, String> {
    let mut config = read_collections(&workspace_id)?;

    // 使用 Application 服务生成 ID
    let id = CollectionApplicationService::generate_collection_id();

    let collection = Collection {
        id: id.clone(),
        name,
        description,
        item_type: "collection".to_string(),
        children: Vec::new(),
        method: None,
        url: None,
        params: None,
        headers: None,
        body: None,
        body_type: None,
        form_fields: None,
        saved_responses: None,
        common_headers: None,
        collection_variables: None,
        ws_config: None,
    };

    if let Some(pid) = parent_id {
        let parent_depth = CollectionApplicationService::get_depth(&config, &pid).unwrap_or(0);
        if parent_depth >= 1 {
            return Err("集合最多两层子集合（总共三层），无法在当前层级创建子集合".to_string());
        }

        if let Some(parent) = CollectionApplicationService::find_item_mut(&mut config, &pid) {
            parent.children.push(collection.clone());
        } else {
            return Err("父集合不存在".to_string());
        }
    } else {
        config.collections.push(collection.clone());
    }

    write_single_item(&workspace_id, &collection)?;
    update_index_only(&workspace_id, &config)?;
    Ok(collection)
}

#[tauri::command]
pub fn create_api(
    workspace_id: String,
    name: String,
    method: String,
    url: String,
    parent_id: Option<String>,
    headers: Option<Vec<Header>>,
    body: Option<String>,
    body_type: Option<String>,
    form_fields: Option<Vec<FormField>>,
) -> Result<Collection, String> {
    let mut config = read_collections(&workspace_id)?;

    // 使用 Application 服务生成 ID
    let id = CollectionApplicationService::generate_api_id();

    let api = Collection {
        id: id.clone(),
        name,
        description: None,
        item_type: "api".to_string(),
        children: Vec::new(),
        method: Some(method),
        url: Some(url),
        params: None,
        headers: headers.or_else(|| {
            Some(vec![Header {
                key: "Content-Type".to_string(),
                value: "application/json".to_string(),
                enabled: true,
                description: None,
            }])
        }),
        body: body.or_else(|| Some(String::new())),
        body_type: body_type.or_else(|| Some("raw".to_string())),
        form_fields,
        saved_responses: None,
        common_headers: None,
        collection_variables: None,
        ws_config: None,
    };

    if let Some(pid) = parent_id {
        if let Some(parent) = CollectionApplicationService::find_item_mut(&mut config, &pid) {
            parent.children.push(api.clone());
        } else {
            return Err("父集合不存在".to_string());
        }
    } else {
        config.collections.push(api.clone());
    }

    write_single_item(&workspace_id, &api)?;
    update_index_only(&workspace_id, &config)?;
    Ok(api)
}

#[tauri::command]
pub fn update_api(
    workspace_id: String,
    id: String,
    name: Option<String>,
    method: Option<String>,
    url: Option<String>,
    params: Option<Vec<Param>>,
    headers: Option<Vec<Header>>,
    body: Option<String>,
    body_type: Option<String>,
    form_fields: Option<Vec<FormField>>,
) -> Result<(), String> {
    let mut config = read_collections(&workspace_id)?;

    if let Some(api) = CollectionApplicationService::find_item_mut(&mut config, &id) {
        if api.item_type != "api" {
            return Err("该项不是 API".to_string());
        }
        if let Some(n) = name {
            api.name = n;
        }
        if let Some(m) = method {
            api.method = Some(m);
        }
        if let Some(u) = url {
            api.url = Some(u);
        }
        if let Some(p) = params {
            api.params = Some(p);
        }
        if let Some(h) = headers {
            api.headers = Some(h);
        }
        if let Some(b) = body {
            api.body = Some(b);
        }
        if let Some(bt) = body_type {
            api.body_type = Some(bt);
        }
        if let Some(ff) = form_fields {
            api.form_fields = Some(ff);
        }

        write_single_item_with_index_update(&workspace_id, api)?;
    } else {
        return Err("API 不存在".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn delete_collection_item(workspace_id: String, id: String) -> Result<(), String> {
    let mut config = read_collections(&workspace_id)?;

    let item =
        CollectionApplicationService::find_item(&config, &id).ok_or("该项不存在".to_string())?;

    delete_item_files_recursive(&workspace_id, &item)?;

    CollectionApplicationService::remove_item(&mut config, &id);

    update_index_only(&workspace_id, &config)?;
    Ok(())
}

#[tauri::command]
pub fn update_collection(
    workspace_id: String,
    id: String,
    name: String,
    description: Option<String>,
) -> Result<(), String> {
    let mut config = read_collections(&workspace_id)?;

    if let Some(col) = CollectionApplicationService::find_item_mut(&mut config, &id) {
        if col.item_type != "collection" {
            return Err("该项不是集合".to_string());
        }
        col.name = name;
        col.description = description;

        write_single_item_with_index_update(&workspace_id, col)?;
    } else {
        return Err("集合不存在".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn update_collection_settings(
    workspace_id: String,
    id: String,
    common_headers: Option<Vec<Header>>,
    collection_variables: Option<Vec<Variable>>,
) -> Result<(), String> {
    let mut config = read_collections(&workspace_id)?;

    if let Some(col) = CollectionApplicationService::find_item_mut(&mut config, &id) {
        if col.item_type != "collection" {
            return Err("该项不是集合".to_string());
        }
        col.common_headers = common_headers;
        col.collection_variables = collection_variables;

        write_single_item(&workspace_id, col)?;
    } else {
        return Err("集合不存在".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn reorder_collection_items(
    workspace_id: String,
    parent_id: Option<String>,
    item_id: String,
    new_index: usize,
) -> Result<(), String> {
    let mut config = read_collections(&workspace_id)?;

    let children =
        CollectionApplicationService::find_parent_children_mut(&mut config, parent_id.as_deref())
            .ok_or("父集合不存在")?;

    let current_index = children
        .iter()
        .position(|item| item.id == item_id)
        .ok_or("项不存在")?;

    if current_index == new_index {
        return Ok(());
    }

    let item = children.remove(current_index);
    let insert_index = if current_index < new_index {
        new_index.saturating_sub(1).min(children.len())
    } else {
        new_index.min(children.len())
    };
    children.insert(insert_index, item);

    update_index_only(&workspace_id, &config)?;
    Ok(())
}

#[tauri::command]
pub fn move_api(
    workspace_id: String,
    api_id: String,
    target_collection_id: Option<String>,
) -> Result<(), String> {
    let mut config = read_collections(&workspace_id)?;

    let api = if let Some(found_api) = CollectionApplicationService::find_api(&config, &api_id) {
        let cloned = found_api.clone();
        CollectionApplicationService::remove_item(&mut config, &api_id);
        cloned
    } else {
        return Err("API 不存在".to_string());
    };

    if api.item_type != "api" {
        return Err("只能移动 API".to_string());
    }

    if let Some(target_id) = target_collection_id {
        let target_depth =
            CollectionApplicationService::get_depth(&config, &target_id).unwrap_or(0);
        if target_depth >= 2 {
            return Err("集合最多三层，无法移动到更深层".to_string());
        }

        if let Some(target) = CollectionApplicationService::find_item_mut(&mut config, &target_id) {
            if target.item_type != "collection" {
                return Err("目标不是集合".to_string());
            }
            target.children.push(api);
        } else {
            return Err("目标集合不存在".to_string());
        }
    } else {
        config.collections.push(api);
    }

    update_index_only(&workspace_id, &config)?;
    Ok(())
}

#[tauri::command]
pub fn move_collection(
    workspace_id: String,
    collection_id: String,
    target_collection_id: Option<String>,
) -> Result<(), String> {
    let mut config = read_collections(&workspace_id)?;

    if let Some(ref target_id) = target_collection_id {
        let descendants = CollectionApplicationService::get_descendant_ids(&config, &collection_id)
            .unwrap_or_default();
        if descendants.contains(target_id) {
            return Err("不能移动到自己的子集合".to_string());
        }
    }

    if target_collection_id.as_ref() == Some(&collection_id) {
        return Err("不能移动到自己".to_string());
    }

    let source_max_child_depth =
        CollectionApplicationService::get_max_child_depth(&config, &collection_id).unwrap_or(0);

    let target_depth = if let Some(ref target_id) = target_collection_id {
        CollectionApplicationService::get_depth(&config, target_id).unwrap_or(0)
    } else {
        0
    };

    let new_max_depth = target_depth + 1 + source_max_child_depth;
    if new_max_depth > 2 {
        return Err(format!(
            "移动后层级超过限制（最多三层），当前将达到 {} 层",
            new_max_depth + 1
        ));
    }

    let collection = if let Some(found) =
        CollectionApplicationService::find_item_mut(&mut config, &collection_id)
    {
        let cloned = found.clone();
        CollectionApplicationService::remove_item(&mut config, &collection_id);
        cloned
    } else {
        return Err("集合不存在".to_string());
    };

    if collection.item_type != "collection" {
        return Err("只能移动集合".to_string());
    }

    if let Some(ref target_id) = target_collection_id {
        if let Some(target) = CollectionApplicationService::find_item_mut(&mut config, target_id) {
            if target.item_type != "collection" {
                return Err("目标不是集合".to_string());
            }
            target.children.push(collection);
        } else {
            return Err("目标集合不存在".to_string());
        }
    } else {
        config.collections.push(collection);
    }

    update_index_only(&workspace_id, &config)?;
    Ok(())
}

fn add_copy_suffix(name: &str) -> String {
    let has_chinese = name.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
    if has_chinese {
        format!("{} 副本", name)
    } else {
        format!("{} Copy", name)
    }
}

fn generate_new_id(item_type: &str, _counter: u32) -> String {
    let prefix = if item_type == "collection" {
        "col"
    } else {
        "api"
    };
    crate::domain::models::common::generate_id(prefix)
}

fn duplicate_scripts_for_item(
    workspace_id: &str,
    old_id: &str,
    new_id: &str,
    item_type: &str,
) -> Result<(), String> {
    let config = ScriptApplicationService::get_config(workspace_id)?;

    let target_type = if item_type == "collection" {
        ScriptTargetType::Collection
    } else {
        ScriptTargetType::Api
    };

    let old_scripts: Vec<_> = config
        .scripts
        .iter()
        .filter(|s| s.target_type == target_type && s.target_id.as_deref() == Some(old_id))
        .cloned()
        .collect();

    if old_scripts.is_empty() {
        return Ok(());
    }

    let mut new_config = ScriptApplicationService::get_config(workspace_id)?;

    for old_script in &old_scripts {
        let old_filename = old_script.file.replace("scripts/", "");
        let content = ScriptApplicationService::read_file(workspace_id, &old_filename)?;

        let new_filename = ScriptApplicationService::generate_filename(
            target_type.clone(),
            Some(new_id),
            old_script.script_kind.clone(),
        );

        ScriptApplicationService::write_file(workspace_id, &new_filename, &content)?;

        let new_entry = ScriptIndexEntry {
            target_type: target_type.clone(),
            target_id: Some(new_id.to_string()),
            script_kind: old_script.script_kind.clone(),
            file: format!("scripts/{}", new_filename),
        };

        new_config.scripts.push(new_entry);
    }

    ScriptApplicationService::save_config(workspace_id, &new_config)?;

    Ok(())
}

fn duplicate_scripts_recursive(
    workspace_id: &str,
    original_item: &crate::domain::models::Collection,
    duplicated_item: &crate::domain::models::Collection,
) -> Result<(), String> {
    duplicate_scripts_for_item(
        workspace_id,
        &original_item.id,
        &duplicated_item.id,
        &original_item.item_type,
    )?;

    for (orig_child, dup_child) in original_item
        .children
        .iter()
        .zip(duplicated_item.children.iter())
    {
        duplicate_scripts_recursive(workspace_id, orig_child, dup_child)?;
    }

    Ok(())
}

fn duplicate_item_recursive(
    item: &crate::domain::models::Collection,
    counter: &mut u32,
    is_root: bool,
) -> crate::domain::models::Collection {
    let new_id = generate_new_id(&item.item_type, *counter);
    *counter += 1;
    let new_name = if is_root {
        add_copy_suffix(&item.name)
    } else {
        item.name.clone()
    };

    let new_children: Vec<crate::domain::models::Collection> = item
        .children
        .iter()
        .map(|child| duplicate_item_recursive(child, counter, false))
        .collect();

    crate::domain::models::Collection {
        id: new_id,
        name: new_name,
        description: item.description.clone(),
        item_type: item.item_type.clone(),
        children: new_children,
        method: item.method.clone(),
        url: item.url.clone(),
        params: item.params.clone(),
        headers: item.headers.clone(),
        body: item.body.clone(),
        body_type: item.body_type.clone(),
        form_fields: item.form_fields.clone(),
        saved_responses: None,
        common_headers: item.common_headers.clone(),
        collection_variables: item.collection_variables.clone(),
        ws_config: item.ws_config.clone(),
    }
}

#[tauri::command]
pub fn duplicate_api(
    workspace_id: String,
    api_id: String,
) -> Result<crate::domain::models::Collection, String> {
    let mut config = read_collections(&workspace_id)?;

    let original_api =
        CollectionApplicationService::find_api(&config, &api_id).ok_or("API 不存在".to_string())?;

    if original_api.item_type != "api" {
        return Err("该项不是 API".to_string());
    }

    let mut counter = 0u32;
    let duplicated = duplicate_item_recursive(&original_api, &mut counter, true);

    let (parent_id, current_index) = find_parent_and_index(&config.collections, &api_id)?;

    if let Some(pid) = parent_id {
        if let Some(parent) = CollectionApplicationService::find_item_mut(&mut config, &pid) {
            parent
                .children
                .insert(current_index + 1, duplicated.clone());
        } else {
            return Err("父集合不存在".to_string());
        }
    } else {
        config
            .collections
            .insert(current_index + 1, duplicated.clone());
    }

    write_single_item(&workspace_id, &duplicated)?;
    update_index_only(&workspace_id, &config)?;

    duplicate_scripts_for_item(&workspace_id, &api_id, &duplicated.id, "api")?;

    Ok(duplicated)
}

#[tauri::command]
pub fn duplicate_collection(
    workspace_id: String,
    collection_id: String,
) -> Result<crate::domain::models::Collection, String> {
    let mut config = read_collections(&workspace_id)?;

    let original = CollectionApplicationService::find_item(&config, &collection_id)
        .ok_or("集合不存在".to_string())?;

    if original.item_type != "collection" {
        return Err("该项不是集合".to_string());
    }

    let original_clone = original.clone();
    let mut counter = 0u32;
    let duplicated = duplicate_item_recursive(&original, &mut counter, true);

    let (parent_id, current_index) = find_parent_and_index(&config.collections, &collection_id)?;

    if let Some(pid) = parent_id {
        if let Some(parent) = CollectionApplicationService::find_item_mut(&mut config, &pid) {
            parent
                .children
                .insert(current_index + 1, duplicated.clone());
        } else {
            return Err("父集合不存在".to_string());
        }
    } else {
        config
            .collections
            .insert(current_index + 1, duplicated.clone());
    }

    write_items_recursive_for_duplicate(&workspace_id, &duplicated)?;
    update_index_only(&workspace_id, &config)?;

    duplicate_scripts_recursive(&workspace_id, &original_clone, &duplicated)?;

    Ok(duplicated)
}

fn write_items_recursive_for_duplicate(
    workspace_id: &str,
    item: &crate::domain::models::Collection,
) -> Result<(), String> {
    write_single_item(workspace_id, item)?;

    for child in &item.children {
        write_items_recursive_for_duplicate(workspace_id, child)?;
    }

    Ok(())
}

fn find_parent_and_index(
    collections: &[crate::domain::models::Collection],
    item_id: &str,
) -> Result<(Option<String>, usize), String> {
    for (index, item) in collections.iter().enumerate() {
        if item.id == item_id {
            return Ok((None, index));
        }
        if item.item_type == "collection" {
            let result = find_parent_and_index_in_children(&item.children, item_id, &item.id);
            if let Some((pid, idx)) = result {
                return Ok((Some(pid), idx));
            }
        }
    }
    Err("找不到项".to_string())
}

fn find_parent_and_index_in_children(
    children: &[crate::domain::models::Collection],
    item_id: &str,
    parent_id: &str,
) -> Option<(String, usize)> {
    for (index, child) in children.iter().enumerate() {
        if child.id == item_id {
            return Some((parent_id.to_string(), index));
        }
        if child.item_type == "collection" {
            let result = find_parent_and_index_in_children(&child.children, item_id, &child.id);
            if let Some((pid, idx)) = result {
                return Some((pid, idx));
            }
        }
    }
    None
}

/// 验证集合名称
#[tauri::command]
pub fn validate_collection_name(name: String) -> Result<(), String> {
    crate::application::services::CollectionApplicationService::validate_collection_name(&name)
}

/// 使用 Collection Application Service 创建集合（示例）
#[tauri::command]
pub fn create_collection_with_service(
    name: String,
    description: Option<String>,
) -> Result<Collection, String> {
    crate::application::services::CollectionApplicationService::create_collection(name, description)
}
