//! 脚本执行命令
//!
//! 提供脚本执行的 Tauri 命令接口。

use crate::application::services::ScriptExecutionApplicationService;
use crate::domain::models::{
    ExecutePostScriptsInput, ExecutePreScriptsInput, ScriptExecutionResult,
};
use tauri::{command, AppHandle};

/// 执行前置脚本链
#[command]
pub async fn execute_pre_scripts_cmd(
    app: AppHandle,
    input: ExecutePreScriptsInput,
) -> Result<ScriptExecutionResult, String> {
    let result = ScriptExecutionApplicationService::execute_pre_scripts(
        app,
        &input.workspace_id,
        input.api_id.as_deref(),
        input.environment_id.as_deref(),
        input.ancestor_collections,
        input.environment_variables,
        input.collection_variables,
        input.request,
        input.silent,
    )
    .await;

    Ok(result)
}

/// 执行后置脚本链
#[command]
pub async fn execute_post_scripts_cmd(
    app: AppHandle,
    input: ExecutePostScriptsInput,
) -> Result<ScriptExecutionResult, String> {
    let result = ScriptExecutionApplicationService::execute_post_scripts(
        app,
        &input.workspace_id,
        input.api_id.as_deref(),
        input.environment_id.as_deref(),
        input.ancestor_collections,
        input.environment_variables,
        input.collection_variables,
        input.request,
        input.response,
        input.silent,
    )
    .await;

    Ok(result)
}
