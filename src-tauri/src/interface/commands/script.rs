use crate::application::services::ScriptApplicationService;
use crate::domain::models::{ScriptIndexEntry, ScriptKind, ScriptTargetType};
use tauri::command;

/// 保存脚本
/// target_type: "api" | "collection" | "workspace" | "environment"
/// script_kind: "pre" | "post"
#[command]
pub async fn save_script(
    workspace_id: String,
    target_type: String,
    target_id: Option<String>,
    script_kind: String,
    content: String,
) -> Result<(), String> {
    // 解析类型
    let target_type_enum = match target_type.as_str() {
        "api" => ScriptTargetType::Api,
        "collection" => ScriptTargetType::Collection,
        "workspace" => ScriptTargetType::Workspace,
        "environment" => ScriptTargetType::Environment,
        _ => return Err(format!("无效的脚本目标类型: {}", target_type)),
    };

    // 解析种类
    let script_kind_enum = match script_kind.as_str() {
        "pre" => ScriptKind::Pre,
        "post" => ScriptKind::Post,
        _ => return Err(format!("无效的脚本种类: {}", script_kind)),
    };

    ScriptApplicationService::save(
        &workspace_id,
        target_type_enum,
        target_id,
        script_kind_enum,
        &content,
    )
}

/// 获取脚本内容
#[command]
pub async fn get_script(
    workspace_id: String,
    target_type: String,
    target_id: Option<String>,
    script_kind: String,
) -> Result<String, String> {
    // 解析类型
    let target_type_enum = match target_type.as_str() {
        "api" => ScriptTargetType::Api,
        "collection" => ScriptTargetType::Collection,
        "workspace" => ScriptTargetType::Workspace,
        "environment" => ScriptTargetType::Environment,
        _ => return Err(format!("无效的脚本目标类型: {}", target_type)),
    };

    // 解析种类
    let script_kind_enum = match script_kind.as_str() {
        "pre" => ScriptKind::Pre,
        "post" => ScriptKind::Post,
        _ => return Err(format!("无效的脚本种类: {}", script_kind)),
    };

    ScriptApplicationService::get(&workspace_id, target_type_enum, target_id, script_kind_enum)
}

/// 删除脚本
#[command]
pub async fn delete_script(
    workspace_id: String,
    target_type: String,
    target_id: Option<String>,
    script_kind: String,
) -> Result<(), String> {
    // 解析类型
    let target_type_enum = match target_type.as_str() {
        "api" => ScriptTargetType::Api,
        "collection" => ScriptTargetType::Collection,
        "workspace" => ScriptTargetType::Workspace,
        "environment" => ScriptTargetType::Environment,
        _ => return Err(format!("无效的脚本目标类型: {}", target_type)),
    };

    // 解析种类
    let script_kind_enum = match script_kind.as_str() {
        "pre" => ScriptKind::Pre,
        "post" => ScriptKind::Post,
        _ => return Err(format!("无效的脚本种类: {}", script_kind)),
    };

    ScriptApplicationService::delete(&workspace_id, target_type_enum, target_id, script_kind_enum)
}

/// 删除目标的所有脚本（删除 api/collection 时调用）
#[command]
pub async fn delete_target_scripts(
    workspace_id: String,
    target_type: String,
    target_id: Option<String>,
) -> Result<(), String> {
    // 解析类型
    let target_type_enum = match target_type.as_str() {
        "api" => ScriptTargetType::Api,
        "collection" => ScriptTargetType::Collection,
        "workspace" => ScriptTargetType::Workspace,
        "environment" => ScriptTargetType::Environment,
        _ => return Err(format!("无效的脚本目标类型: {}", target_type)),
    };

    ScriptApplicationService::delete_by_target(&workspace_id, target_type_enum, target_id)
}

/// 获取所有脚本列表
#[command]
pub async fn get_all_scripts(workspace_id: String) -> Result<Vec<ScriptIndexEntry>, String> {
    ScriptApplicationService::get_all(&workspace_id)
}
