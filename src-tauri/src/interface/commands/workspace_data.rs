//! 工作区数据命令
//!
//! environment / memory / cookie 的 Tauri 命令。
//!
//! ## 设计理由
//! - 三者的持久化操作在 WorkspaceDataRepository 中独立读写
//! - Tauri 命令合并，保持架构一致性

use crate::application::services::{
    read_collections, replace_variables, CollectionApplicationService,
    WorkspaceDataApplicationService,
};
use crate::domain::models::{
    Collection, Cookie, Environment, EnvironmentsConfig, Variable, VariableInfo,
};
use std::collections::HashMap;

// === Environment 命令 ===

/// 获取所有环境
#[tauri::command]
pub fn get_environments(workspace_id: String) -> Result<EnvironmentsConfig, String> {
    WorkspaceDataApplicationService::read_environments(&workspace_id)
}

/// 保存环境（创建或更新）
#[tauri::command]
pub fn save_environment(
    workspace_id: String,
    environment: Environment,
) -> Result<Environment, String> {
    let mut config = WorkspaceDataApplicationService::read_environments(&workspace_id)?;

    // 查找是否存在相同 id 的环境
    let existing = config
        .environments
        .iter_mut()
        .find(|e| e.id == environment.id);

    if let Some(env) = existing {
        // 更新现有环境
        *env = environment.clone();
    } else {
        // 创建新环境，不自动设置为激活环境
        config.environments.push(environment.clone());
    }

    WorkspaceDataApplicationService::write_environments(&workspace_id, &config)?;
    Ok(environment)
}

/// 删除环境
#[tauri::command]
pub fn delete_environment(workspace_id: String, environment_id: String) -> Result<(), String> {
    let mut config = WorkspaceDataApplicationService::read_environments(&workspace_id)?;

    config.environments.retain(|e| e.id != environment_id);

    // 如果删除的是当前激活的环境，清除激活状态或切换到第一个环境
    if config.active_environment_id == Some(environment_id) {
        config.active_environment_id = config.environments.first().map(|e| e.id.clone());
    }

    WorkspaceDataApplicationService::write_environments(&workspace_id, &config)?;
    Ok(())
}

/// 切换激活环境
#[tauri::command]
pub fn switch_environment(workspace_id: String, environment_id: String) -> Result<(), String> {
    let mut config = WorkspaceDataApplicationService::read_environments(&workspace_id)?;

    // 验证环境是否存在
    if !config.environments.iter().any(|e| e.id == environment_id) {
        return Err("环境不存在".to_string());
    }

    config.active_environment_id = Some(environment_id);
    WorkspaceDataApplicationService::write_environments(&workspace_id, &config)?;
    Ok(())
}

/// 获取当前激活环境的变量映射
#[tauri::command]
pub fn get_active_variables(workspace_id: String) -> Result<HashMap<String, String>, String> {
    let config = WorkspaceDataApplicationService::read_environments(&workspace_id)?;
    Ok(WorkspaceDataApplicationService::get_active_variables_map(
        &config,
    ))
}

/// 环境排序
#[tauri::command]
pub fn reorder_environments(
    workspace_id: String,
    environment_id: String,
    new_index: usize,
) -> Result<(), String> {
    let mut config = WorkspaceDataApplicationService::read_environments(&workspace_id)?;

    // 查找环境当前位置
    let current_index = config
        .environments
        .iter()
        .position(|e| e.id == environment_id)
        .ok_or_else(|| "环境不存在".to_string())?;

    // 移动环境到新位置
    if current_index != new_index {
        let environment = config.environments.remove(current_index);
        // 确保新索引在有效范围内
        let insert_index = new_index.min(config.environments.len());
        config.environments.insert(insert_index, environment);
        WorkspaceDataApplicationService::write_environments(&workspace_id, &config)?;
    }

    Ok(())
}

/// 获取可用的变量列表（用于前端变量提示）
#[tauri::command]
pub fn get_available_variables(
    workspace_id: String,
    environment_id: Option<String>,
    item_id: String,
    _item_type: String,
) -> Result<Vec<VariableInfo>, String> {
    let mut variables_map: HashMap<String, VariableInfo> = HashMap::new();

    let env_config = WorkspaceDataApplicationService::read_environments(&workspace_id)?;
    let target_env_id = environment_id.or(env_config.active_environment_id);

    if let Some(env_id) = target_env_id {
        if let Some(env) = env_config.environments.iter().find(|e| e.id == env_id) {
            for v in &env.variables {
                if v.enabled {
                    variables_map.insert(
                        v.key.clone(),
                        VariableInfo {
                            key: v.key.clone(),
                            value: v.value.clone(),
                            source: env.name.clone(),
                            description: v.description.clone(),
                        },
                    );
                }
            }
        }
    }

    // 2. 集合变量
    let collections_config = read_collections(&workspace_id)?;
    let mut ancestor_chain: Vec<Collection> = Vec::new();

    if CollectionApplicationService::find_ancestor_chain(
        &collections_config,
        &item_id,
        &mut ancestor_chain,
    ) {
        for collection in ancestor_chain.iter() {
            if collection.item_type != "collection" {
                continue;
            }
            if let Some(collection_vars) = &collection.collection_variables {
                for v in collection_vars {
                    if v.enabled {
                        variables_map.insert(
                            v.key.clone(),
                            VariableInfo {
                                key: v.key.clone(),
                                value: v.value.clone(),
                                source: collection.name.clone(),
                                description: v.description.clone(),
                            },
                        );
                    }
                }
            }
        }
    }

    Ok(variables_map.values().cloned().collect())
}

/// 替换文本中的变量
#[tauri::command]
pub fn replace_variables_text(text: String, variables: Vec<Variable>) -> String {
    let vars_map: HashMap<String, String> = variables
        .iter()
        .filter(|v| v.enabled && !v.key.is_empty())
        .map(|v| (v.key.clone(), v.value.clone()))
        .collect();

    replace_variables(&text, &vars_map).text
}

/// 验证环境名称
#[tauri::command]
pub fn validate_environment_name(name: String) -> Result<(), String> {
    WorkspaceDataApplicationService::validate_variable_key(&name)
}

/// 创建环境
#[tauri::command]
pub fn create_environment_with_service(name: String) -> Result<Environment, String> {
    WorkspaceDataApplicationService::create_environment(name)
}

// === Memory 命令 ===

/// 获取展开的集合ID列表
#[tauri::command]
pub fn get_expanded_collections(workspace_id: String) -> Result<Vec<String>, String> {
    WorkspaceDataApplicationService::get_expanded_collections(&workspace_id)
}

/// 保存展开的集合ID列表
#[tauri::command]
pub fn save_expanded_collections(
    workspace_id: String,
    expanded_ids: Vec<String>,
) -> Result<(), String> {
    WorkspaceDataApplicationService::save_expanded_collections(&workspace_id, expanded_ids)
}

/// 获取打开的标签页（返回: open_tabs, open_tab_types, active_tab_index, request_tabs）
#[tauri::command]
pub fn get_open_tabs(
    workspace_id: String,
) -> Result<(Vec<String>, Vec<String>, usize, HashMap<String, String>), String> {
    WorkspaceDataApplicationService::get_open_tabs(&workspace_id)
}

/// 保存打开的标签页
#[tauri::command]
pub fn save_open_tabs(
    workspace_id: String,
    open_tabs: Vec<String>,
    open_tab_types: Vec<String>,
    active_tab_index: usize,
    request_tabs: HashMap<String, String>,
) -> Result<(), String> {
    WorkspaceDataApplicationService::save_open_tabs(
        &workspace_id,
        open_tabs,
        open_tab_types,
        active_tab_index,
        request_tabs,
    )
}

// === Cookie 命令 ===

/// 获取所有 Cookie
#[tauri::command]
pub fn get_cookies(workspace_id: String) -> Result<Vec<Cookie>, String> {
    WorkspaceDataApplicationService::get_cookies(&workspace_id)
}

/// 清除所有 Cookie
#[tauri::command]
pub fn clear_cookies(workspace_id: String) -> Result<(), String> {
    WorkspaceDataApplicationService::clear_cookies(&workspace_id)
}

/// 删除指定 Cookie
#[tauri::command]
pub fn delete_cookie(workspace_id: String, name: String, domain: String) -> Result<(), String> {
    WorkspaceDataApplicationService::delete_cookie(&workspace_id, &name, &domain)
}

/// 添加或更新 Cookie
#[tauri::command]
pub fn add_cookie(workspace_id: String, cookie: Cookie) -> Result<(), String> {
    WorkspaceDataApplicationService::add_cookie(&workspace_id, &cookie)
}
