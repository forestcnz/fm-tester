//! 脚本执行应用服务
//!
//! 协调脚本链加载、执行、日志发送和变量保存。

use crate::application::services::ScriptApplicationService;
use crate::domain::models::{
    CollectionInfo, ScriptExecutionContext, ScriptExecutionResult, ScriptInfo, ScriptKind,
    ScriptRequestContext, ScriptResponseContext, ScriptTargetType,
};
use crate::domain::services::ScriptExecutionDomainService;
use crate::infrastructure::JsRuntimeExecutor;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

/// 脚本执行应用服务
pub struct ScriptExecutionApplicationService;

impl ScriptExecutionApplicationService {
    /// 执行前置脚本链
    pub async fn execute_pre_scripts(
        app: AppHandle,
        workspace_id: &str,
        api_id: Option<&str>,
        environment_id: Option<&str>,
        ancestor_collections: Vec<CollectionInfo>,
        environment_variables: HashMap<String, String>,
        collection_variables: HashMap<String, String>,
        request: ScriptRequestContext,
        silent: bool,
    ) -> ScriptExecutionResult {
        tracing::debug!(
            "执行前置脚本链: api_id={:?}, environment_id={:?}, collections={}",
            api_id,
            environment_id,
            ancestor_collections.len()
        );
        // 构建 all_collection_variables（按集合 ID 分组）
        let all_collection_variables: HashMap<String, HashMap<String, String>> =
            ancestor_collections
                .iter()
                .map(|c| {
                    let vars: HashMap<String, String> = c
                        .collection_variables
                        .iter()
                        .filter(|v| v.enabled)
                        .map(|v| (v.key.clone(), v.value.clone()))
                        .collect();
                    (c.id.clone(), vars)
                })
                .collect();

        // 获取父集合 ID（API 的直接父）
        let parent_collection_id = ancestor_collections.last().map(|c| c.id.clone());

        // 加载脚本链
        let scripts = Self::load_script_chain(
            workspace_id,
            api_id,
            environment_id,
            &ancestor_collections,
            ScriptKind::Pre,
        );

        if scripts.is_empty() {
            // 无脚本，直接返回原始数据
            return ScriptExecutionResult {
                success: true,
                modified_environment_vars: environment_variables,
                modified_target_environment_vars: None,
                target_environment_id: environment_id.map(|s| s.to_string()),
                modified_collection_vars: collection_variables,
                modified_target_collection_vars: None,
                target_collection_id: parent_collection_id,
                modified_request: Some(request),
                test_results: Vec::new(),
                logs: Vec::new(),
                error: None,
                error_source: None,
            };
        }

        // 执行脚本链
        Self::execute_script_chain(
            app,
            scripts,
            environment_variables,
            environment_id.map(|s| s.to_string()),
            collection_variables,
            all_collection_variables,
            parent_collection_id,
            request,
            None, // 前置脚本没有响应
            ScriptKind::Pre,
            silent,
        )
        .await
    }

    /// 执行后置脚本链
    pub async fn execute_post_scripts(
        app: AppHandle,
        workspace_id: &str,
        api_id: Option<&str>,
        environment_id: Option<&str>,
        ancestor_collections: Vec<CollectionInfo>,
        environment_variables: HashMap<String, String>,
        collection_variables: HashMap<String, String>,
        request: ScriptRequestContext,
        response: ScriptResponseContext,
        silent: bool,
    ) -> ScriptExecutionResult {
        // 构建 all_collection_variables（按集合 ID 分组）
        let all_collection_variables: HashMap<String, HashMap<String, String>> =
            ancestor_collections
                .iter()
                .map(|c| {
                    let vars: HashMap<String, String> = c
                        .collection_variables
                        .iter()
                        .filter(|v| v.enabled)
                        .map(|v| (v.key.clone(), v.value.clone()))
                        .collect();
                    (c.id.clone(), vars)
                })
                .collect();

        // 获取父集合 ID（API 的直接父）
        let parent_collection_id = ancestor_collections.last().map(|c| c.id.clone());

        // 加载脚本链
        let scripts = Self::load_script_chain(
            workspace_id,
            api_id,
            environment_id,
            &ancestor_collections,
            ScriptKind::Post,
        );

        if scripts.is_empty() {
            // 无脚本，直接返回原始数据
            return ScriptExecutionResult {
                success: true,
                modified_environment_vars: environment_variables,
                modified_target_environment_vars: None,
                target_environment_id: environment_id.map(|s| s.to_string()),
                modified_collection_vars: collection_variables,
                modified_target_collection_vars: None,
                target_collection_id: parent_collection_id,
                modified_request: Some(request),
                test_results: Vec::new(),
                logs: Vec::new(),
                error: None,
                error_source: None,
            };
        }

        // 后置脚本反向执行
        let ordered_scripts =
            ScriptExecutionDomainService::order_script_chain(scripts, ScriptKind::Post);

        // 执行脚本链
        Self::execute_script_chain(
            app,
            ordered_scripts,
            environment_variables,
            environment_id.map(|s| s.to_string()),
            collection_variables,
            all_collection_variables,
            parent_collection_id,
            request,
            Some(response),
            ScriptKind::Post,
            silent,
        )
        .await
    }

    /// 加载脚本链
    fn load_script_chain(
        workspace_id: &str,
        api_id: Option<&str>,
        environment_id: Option<&str>,
        ancestor_collections: &[CollectionInfo],
        kind: ScriptKind,
    ) -> Vec<ScriptInfo> {
        let mut scripts = Vec::new();

        // 1. 工作区脚本
        if let Ok(content) = ScriptApplicationService::get(
            workspace_id,
            ScriptTargetType::Workspace,
            None,
            kind.clone(),
        ) {
            if !ScriptExecutionDomainService::is_empty_script(&content) {
                scripts.push(ScriptInfo {
                    source: ScriptExecutionDomainService::generate_source_identifier(
                        ScriptTargetType::Workspace,
                        None,
                        None,
                    ),
                    content,
                    source_type: ScriptTargetType::Workspace,
                    target_id: None,
                });
            }
        }

        // 2. 环境脚本
        if let Some(env_id) = environment_id {
            if let Ok(content) = ScriptApplicationService::get(
                workspace_id,
                ScriptTargetType::Environment,
                Some(env_id.to_string()),
                kind.clone(),
            ) {
                if !ScriptExecutionDomainService::is_empty_script(&content) {
                    scripts.push(ScriptInfo {
                        source: ScriptExecutionDomainService::generate_source_identifier(
                            ScriptTargetType::Environment,
                            None,
                            None,
                        ),
                        content,
                        source_type: ScriptTargetType::Environment,
                        target_id: Some(env_id.to_string()),
                    });
                }
            }
        }

        // 3. 集合脚本（按层级顺序：父 → 子）
        for collection in ancestor_collections {
            if let Ok(content) = ScriptApplicationService::get(
                workspace_id,
                ScriptTargetType::Collection,
                Some(collection.id.clone()),
                kind.clone(),
            ) {
                if !ScriptExecutionDomainService::is_empty_script(&content) {
                    scripts.push(ScriptInfo {
                        source: ScriptExecutionDomainService::generate_source_identifier(
                            ScriptTargetType::Collection,
                            Some(&collection.id),
                            Some(&collection.name),
                        ),
                        content,
                        source_type: ScriptTargetType::Collection,
                        target_id: Some(collection.id.clone()),
                    });
                }
            }
        }

        // 4. 接口脚本
        if let Some(api_id) = api_id {
            if let Ok(content) = ScriptApplicationService::get(
                workspace_id,
                ScriptTargetType::Api,
                Some(api_id.to_string()),
                kind.clone(),
            ) {
                if !ScriptExecutionDomainService::is_empty_script(&content) {
                    scripts.push(ScriptInfo {
                        source: ScriptExecutionDomainService::generate_source_identifier(
                            ScriptTargetType::Api,
                            None,
                            None,
                        ),
                        content,
                        source_type: ScriptTargetType::Api,
                        target_id: Some(api_id.to_string()),
                    });
                }
            }
        }

        scripts
    }

    /// 执行脚本链
    async fn execute_script_chain(
        app: AppHandle,
        scripts: Vec<ScriptInfo>,
        environment_variables: HashMap<String, String>,
        target_environment_id: Option<String>,
        collection_variables: HashMap<String, String>,
        all_collection_variables: HashMap<String, HashMap<String, String>>,
        parent_collection_id: Option<String>,
        request: ScriptRequestContext,
        response: Option<ScriptResponseContext>,
        kind: ScriptKind,
        silent: bool,
    ) -> ScriptExecutionResult {
        let mut env_vars = environment_variables;
        let mut target_env_vars: Option<HashMap<String, String>> =
            if target_environment_id.is_some() {
                Some(env_vars.clone())
            } else {
                None
            };
        let mut coll_vars = collection_variables;
        let mut target_coll_vars: Option<HashMap<String, String>> = None;
        let mut last_target_collection_id: Option<String> = None;
        let mut all_coll_vars = all_collection_variables;
        let mut req = request;
        let mut all_logs = Vec::new();
        let mut all_test_results = Vec::new();
        let mut errors = Vec::new();

        for script in scripts {
            // 根据脚本类型确定目标集合
            let (target_collection_id, is_api_script) = match script.source_type {
                ScriptTargetType::Collection => (script.target_id.clone(), false),
                ScriptTargetType::Api => (parent_collection_id.clone(), true),
                _ => (None, false),
            };

            // 初始化目标集合变量
            if target_collection_id.is_some() && target_coll_vars.is_none() {
                target_coll_vars = target_collection_id
                    .clone()
                    .and_then(|id| all_coll_vars.get(&id).cloned());
            }

            // 构建执行上下文
            let context = ScriptExecutionContext {
                environment_variables: env_vars.clone(),
                collection_variables: coll_vars.clone(),
                all_collection_variables: all_coll_vars.clone(),
                target_collection_id: target_collection_id.clone(),
                target_environment_id: target_environment_id.clone(),
                is_api_script,
                parent_collection_id: parent_collection_id.clone(),
                request: req.clone(),
                response: response.clone(),
            };

            last_target_collection_id = target_collection_id.clone();

            // 执行单个脚本
            let result =
                JsRuntimeExecutor::execute_script(&script.content, &context, &script.source).await;

            match result {
                Ok(exec_result) => {
                    // 发送日志到前端
                    for log in &exec_result.logs {
                        Self::emit_script_log(&app, log, silent);
                        all_logs.push(log.clone());
                    }

                    // 收集测试结果
                    all_test_results.extend(exec_result.test_results.clone());

                    if !exec_result.success {
                        // 处理错误
                        let error_msg = exec_result.error.clone().unwrap_or_default();
                        let source = exec_result
                            .error_source
                            .clone()
                            .unwrap_or(script.source.clone());

                        // 发送错误日志
                        Self::emit_script_log(
                            &app,
                            &crate::domain::models::ScriptLog {
                                level: "error".to_string(),
                                message: format!("[{}] 执行失败: {}", source, error_msg),
                                source: source.clone(),
                            },
                            silent,
                        );

                        if kind == ScriptKind::Pre {
                            // 前置脚本错误中断执行
                            return ScriptExecutionResult {
                                success: false,
                                modified_environment_vars: env_vars,
                                modified_target_environment_vars: target_env_vars,
                                target_environment_id: target_environment_id.clone(),
                                modified_collection_vars: coll_vars,
                                modified_target_collection_vars: target_coll_vars,
                                target_collection_id: last_target_collection_id,
                                modified_request: Some(req),
                                test_results: all_test_results,
                                logs: all_logs,
                                error: Some(error_msg),
                                error_source: Some(source),
                            };
                        } else {
                            // 后置脚本错误继续执行（记录）
                            errors.push((source, error_msg));
                        }
                    }

                    // 更新上下文（传递给下一个脚本）
                    env_vars = exec_result.modified_environment_vars;
                    target_env_vars = exec_result.modified_target_environment_vars.clone();
                    coll_vars = exec_result.modified_collection_vars;
                    if script.source_type == ScriptTargetType::Collection
                        || script.source_type == ScriptTargetType::Api
                    {
                        target_coll_vars = exec_result.modified_target_collection_vars.clone();
                        // 更新 all_collection_variables 中对应集合的变量
                        if let Some(ref target_id) = exec_result.target_collection_id {
                            if let Some(ref modified_vars) =
                                exec_result.modified_target_collection_vars
                            {
                                all_coll_vars.insert(target_id.clone(), modified_vars.clone());
                            }
                        }
                    }
                    if let Some(modified_req) = exec_result.modified_request {
                        req = modified_req;
                    }
                }
                Err(e) => {
                    // 脚本执行异常
                    Self::emit_script_log(
                        &app,
                        &crate::domain::models::ScriptLog {
                            level: "error".to_string(),
                            message: format!("[{}] 执行异常: {}", script.source, e),
                            source: script.source.clone(),
                        },
                        silent,
                    );

                    if kind == ScriptKind::Pre {
                        return ScriptExecutionResult {
                            success: false,
                            modified_environment_vars: env_vars,
                            modified_target_environment_vars: target_env_vars,
                            target_environment_id: target_environment_id.clone(),
                            modified_collection_vars: coll_vars,
                            modified_target_collection_vars: target_coll_vars,
                            target_collection_id: last_target_collection_id,
                            modified_request: Some(req),
                            test_results: all_test_results,
                            logs: all_logs,
                            error: Some(e),
                            error_source: Some(script.source),
                        };
                    } else {
                        errors.push((script.source.clone(), e));
                    }
                }
            }
        }

        // 构建最终结果
        ScriptExecutionResult {
            success: errors.is_empty(),
            modified_environment_vars: env_vars,
            modified_target_environment_vars: target_env_vars,
            target_environment_id,
            modified_collection_vars: coll_vars,
            modified_target_collection_vars: target_coll_vars,
            target_collection_id: last_target_collection_id,
            modified_request: Some(req),
            test_results: all_test_results,
            logs: all_logs,
            error: if !errors.is_empty() {
                Some(
                    errors
                        .iter()
                        .map(|(s, e)| format!("{}: {}", s, e))
                        .collect::<Vec<_>>()
                        .join(", "),
                )
            } else {
                None
            },
            error_source: None,
        }
    }

    /// 发送脚本日志到前端
    fn emit_script_log(app: &AppHandle, log: &crate::domain::models::ScriptLog, silent: bool) {
        if silent {
            return;
        }

        // 格式化时间
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // 发送事件
        app.emit(
            "script-log",
            serde_json::json!({
                "type": log.level,
                "message": log.message,
                "source": log.source,
                "time": timestamp,
                "level": log.source,
            }),
        )
        .ok();
    }
}
