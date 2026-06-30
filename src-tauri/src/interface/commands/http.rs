//! HTTP 命令接口
//!
//! 提供 HTTP 相关的 Tauri 命令，调用应用服务进行业务处理。

use crate::application::services::{HistoryApplicationService, HttpApplicationService};
use crate::domain::models::{FormField, Header, HttpResponse, Variable};
use tauri::AppHandle;

/// 发送 HTTP 请求
#[tauri::command]
pub async fn send_http_request(
    app: AppHandle,
    method: String,
    url: String,
    headers: Vec<Header>,
    body: Option<String>,
    body_type: Option<String>,
    form_fields: Option<Vec<FormField>>,
    workspace_id: String,
    api_id: Option<String>,
    api_name: Option<String>,
    collection_variables: Option<Vec<Variable>>,
) -> Result<HttpResponse, String> {
    // 调用 Application 服务发送请求
    let response = HttpApplicationService::send_request(
        app.clone(),
        method.clone(),
        url.clone(),
        headers.clone(),
        body.clone(),
        body_type.clone(),
        form_fields.clone(),
        workspace_id.clone(),
        api_id.clone(),
        api_name.clone(),
        collection_variables.clone(),
    )
    .await?;

    // 记录历史（Interface层协调多个服务）
    if let Err(e) = HistoryApplicationService::create_and_save_history(
        &workspace_id,
        method,
        url,
        response.resolved_url.clone(),
        headers,
        body,
        body_type,
        form_fields,
        &response,
        api_id,
        api_name,
    ) {
        eprintln!("记录历史失败: {}", e);
    }

    Ok(response)
}

/// 导出为 curl 命令
#[tauri::command]
pub async fn export_as_curl(
    method: String,
    url: String,
    headers: Vec<Header>,
    body: Option<String>,
    body_type: Option<String>,
    form_fields: Option<Vec<FormField>>,
    workspace_id: String,
    collection_variables: Option<Vec<Variable>>,
) -> Result<String, String> {
    // 获取合并后的变量
    let variables =
        HttpApplicationService::get_merged_variables(&workspace_id, collection_variables.as_ref())?;

    // 替换变量
    let (replaced_url, replaced_headers, replaced_body, _) =
        HttpApplicationService::replace_all_variables(&url, &headers, body.as_deref(), &variables);

    // 构建 curl 命令
    let mut curl_parts: Vec<String> = vec!["curl".to_string()];

    // 添加 method
    let method_upper = method.to_uppercase();
    if method_upper != "GET" {
        curl_parts.push(format!("-X {}", method_upper));
    }

    // 添加 URL
    curl_parts.push(HttpApplicationService::shell_escape(&replaced_url));

    // 添加 Headers
    for header in &replaced_headers {
        if header.enabled && !header.key.trim().is_empty() {
            curl_parts.push(format!(
                "-H {}",
                HttpApplicationService::shell_escape(&format!("{}: {}", header.key, header.value))
            ));
        }
    }

    // 处理请求体
    let actual_body_type = body_type.clone().unwrap_or_else(|| "raw".to_string());

    if method_upper != "GET" && method_upper != "HEAD" {
        match actual_body_type.as_str() {
            "form-data" => {
                if let Some(ref fields) = form_fields {
                    for field in fields {
                        if !field.enabled || field.key.is_empty() {
                            continue;
                        }
                        match field.field_type.as_str() {
                            "text" => {
                                let (replaced_value, _) =
                                    HttpApplicationService::replace_vars(&field.value, &variables);
                                curl_parts.push(format!(
                                    "-F {}",
                                    HttpApplicationService::shell_escape(&format!(
                                        "{}={}",
                                        field.key, replaced_value
                                    ))
                                ));
                            }
                            "file" => {
                                if let Some(ref files) = field.files {
                                    for file_info in files {
                                        // 校验路径非空且无 shell 元字符（防御性，配合 shell_escape 双保险）
                                        let path_str = &file_info.path;
                                        if path_str.trim().is_empty() {
                                            continue;
                                        }
                                        curl_parts.push(format!(
                                            "-F {}",
                                            HttpApplicationService::shell_escape(&format!(
                                                "{}=@{}",
                                                field.key, path_str
                                            ))
                                        ));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {
                // raw 或其他类型
                if let Some(ref b) = replaced_body {
                    if !b.is_empty() {
                        curl_parts.push(format!("-d {}", HttpApplicationService::shell_escape(b)));
                    }
                }
            }
        }
    }

    Ok(curl_parts.join(" "))
}
