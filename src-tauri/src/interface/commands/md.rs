//! 文档命令接口
//!
//! 提供文档相关的 Tauri 命令，处理前端交互。

use crate::application::services::read_collections;
use crate::application::services::{
    AiApplicationService, AppConfigApplicationService, CollectionApplicationService,
    HistoryApplicationService, MdApplicationService, ResponseApplicationService,
};
use crate::domain::models::{
    AiChatMessage, Collection, DocGenerationStatus, DocMetadata, Header, HistoryEntry,
    SavedResponse,
};
use crate::domain::repositories::AiHttpClientService;
use crate::infrastructure::get_ai_http_client;
use tauri::{command, AppHandle, Emitter};

/// 获取 API 文档内容
#[command]
pub fn get_api_doc(workspace_id: String, api_id: String) -> Result<String, String> {
    MdApplicationService::get_doc(&workspace_id, &api_id)
}

/// 保存 API 文档内容
#[command]
pub fn save_api_doc(workspace_id: String, api_id: String, content: String) -> Result<(), String> {
    MdApplicationService::save_doc(&workspace_id, &api_id, &content)
}

/// 获取 API 文档元数据
#[command]
pub fn get_api_doc_metadata(workspace_id: String, api_id: String) -> Result<DocMetadata, String> {
    MdApplicationService::get_metadata(&workspace_id, &api_id)
}

/// 获取文档生成状态
#[command]
pub fn get_doc_generation_status(api_id: String) -> Result<DocGenerationStatus, String> {
    let generating = AiApplicationService::is_generation_running(&api_id);
    let elapsed = AiApplicationService::get_generation_elapsed_seconds(&api_id);

    Ok(DocGenerationStatus {
        api_id,
        generating,
        elapsed_seconds: elapsed,
        error: None,
    })
}

/// 取消文档生成
#[command]
pub fn cancel_doc_generation(api_id: String) -> Result<(), String> {
    AiApplicationService::cancel_generation_task(&api_id);
    Ok(())
}

/// AI 生成 API 文档
#[command]
pub async fn generate_api_doc_with_ai(
    app: AppHandle,
    workspace_id: String,
    api_id: String,
) -> Result<String, String> {
    // 检查是否已有任务正在进行
    if AiApplicationService::is_generation_running(&api_id) {
        return Err("该接口已有生成任务正在进行".to_string());
    }

    // 初始化生成任务状态
    AiApplicationService::init_generation_task(&api_id);

    // 发送开始事件
    app.emit("doc-generation-start", &api_id).ok();

    let result = do_generate_api_doc_async(&app, &workspace_id, &api_id).await;

    // 清理状态
    AiApplicationService::cleanup_generation_task(&api_id);

    // 发送完成事件
    match &result {
        Ok(content) => {
            app.emit("doc-generation-complete", content).ok();
        }
        Err(e) => {
            app.emit("doc-generation-error", e).ok();
        }
    }

    result
}

/// 执行文档生成（异步版本）
async fn do_generate_api_doc_async(
    app: &AppHandle,
    workspace_id: &str,
    api_id: &str,
) -> Result<String, String> {
    // 1. 获取AI配置（已解密）
    let settings_service = AppConfigApplicationService::default();
    let ai_config = settings_service.get_decrypted_ai_config()?;

    if ai_config.encrypted_api_key.is_empty() || ai_config.api_endpoint.is_empty() {
        return Err("请先配置 AI 设置".to_string());
    }

    let decrypted_api_key = ai_config.encrypted_api_key;

    // 2. 获取接口定义
    let collections_config = read_collections(workspace_id)?;
    let api_data = CollectionApplicationService::find_api(&collections_config, api_id)
        .ok_or_else(|| "接口不存在".to_string())?;

    // 3. 获取现有文档
    let existing_doc = MdApplicationService::get_doc(workspace_id, api_id).unwrap_or_default();

    // 4. 获取保存的响应
    let saved_responses_index = ResponseApplicationService::get_by_api(workspace_id, api_id)?;
    let saved_responses: Vec<SavedResponse> = saved_responses_index
        .iter()
        .map(|entry| {
            ResponseApplicationService::get(workspace_id, &entry.id)?
                .ok_or_else(|| format!("响应 {} 不存在", entry.id))
        })
        .collect::<Result<Vec<SavedResponse>, String>>()?;

    // 5. 获取历史记录
    let history_dates = HistoryApplicationService::get_dates(workspace_id)?;
    let recent_dates: Vec<String> = history_dates.into_iter().take(3).collect();
    let mut history_entries: Vec<HistoryEntry> = Vec::new();

    for date in recent_dates {
        if history_entries.len() >= 10 {
            break;
        }
        let entries = HistoryApplicationService::get_by_date(workspace_id, &date)?;
        let api_history: Vec<HistoryEntry> = entries
            .into_iter()
            .filter(|e| e.api_id == Some(api_id.to_string()))
            .collect();
        history_entries.extend(api_history);
    }
    history_entries = history_entries.into_iter().take(10).collect();

    // 6. 构建提示
    let prompt =
        build_doc_generation_prompt(&api_data, &existing_doc, &saved_responses, &history_entries);

    // 7. 发送进度事件
    app.emit("doc-generation-progress", "正在生成文档...").ok();

    // 8. 构建消息
    let messages = vec![
        AiChatMessage::system(
            "你是一个专业的API文档撰写专家。请根据提供的接口信息生成完整、准确的API文档。
文档必须使用Markdown格式，包含完整的请求参数说明和响应参数说明。
特别要求：
1. 请求PATH必须准确，与提供的接口定义一致
2. 响应参数说明必须完整覆盖所有字段，不可遗漏
3. 响应示例必须完整，展示接口的完整返回结构
4. 对于嵌套对象和数组，需要逐层展开说明",
        ),
        AiChatMessage::user(prompt),
    ];

    // 9. 调用AI流式生成
    let client = get_ai_http_client();
    let custom_headers = if ai_config.custom_headers.is_empty() {
        None
    } else {
        Some(ai_config.custom_headers)
    };

    // 使用 api_id 的克隆来创建取消检查闭包
    let api_id_for_cancel = api_id.to_string();
    let cancellation_check =
        Box::new(move || AiApplicationService::is_generation_cancelled(&api_id_for_cancel));

    let content = client
        .chat_stream(
            app.clone(),
            &ai_config.api_endpoint,
            &decrypted_api_key,
            &ai_config.model,
            messages,
            custom_headers,
            ai_config.timeout,
            Some(cancellation_check),
        )
        .await?;

    // 10. 保存文档
    MdApplicationService::save_doc(workspace_id, api_id, &content)?;

    Ok(content)
}

/// 构建文档生成提示
fn build_doc_generation_prompt(
    api_data: &Collection,
    existing_doc: &str,
    saved_responses: &[SavedResponse],
    history_entries: &[HistoryEntry],
) -> String {
    let mut parts = Vec::new();

    // 接口基本信息
    parts.push("# 接口信息".to_string());
    parts.push(format!("- 名称：{}", api_data.name));
    if let Some(desc) = &api_data.description {
        parts.push(format!("- 描述：{}", desc));
    }
    parts.push(format!(
        "- 方法：{}",
        api_data.method.clone().unwrap_or_default()
    ));
    parts.push(format!(
        "- 请求PATH：{}",
        api_data.url.clone().unwrap_or_default()
    ));

    // Headers
    if let Some(headers) = &api_data.headers {
        let enabled_headers: Vec<&Header> = headers.iter().filter(|h| h.enabled).collect();
        if !enabled_headers.is_empty() {
            parts.push("\n## Headers".to_string());
            parts.push("| Header名 | 值 |".to_string());
            parts.push("|----------|------|".to_string());
            for h in enabled_headers {
                parts.push(format!("| {} | {} |", h.key, h.value));
            }
        }
    }

    // Body
    if let Some(body) = &api_data.body {
        if !body.is_empty() {
            parts.push("\n## Body".to_string());
            let body_type = api_data.body_type.as_deref().unwrap_or("raw");
            parts.push(format!("- 类型：{}", body_type));
            parts.push(format!("- 内容：\n{}", body));
        }
    }

    // Form 字段
    if let Some(form_fields) = &api_data.form_fields {
        let enabled_fields: Vec<&crate::domain::models::FormField> =
            form_fields.iter().filter(|f| f.enabled).collect();
        if !enabled_fields.is_empty() {
            parts.push("\n### Form字段".to_string());
            parts.push("| 字段名 | 值 | 类型 |".to_string());
            parts.push("|--------|------|------|".to_string());
            for f in enabled_fields {
                parts.push(format!("| {} | {} | {} |", f.key, f.value, f.field_type));
            }
        }
    }

    // 现有文档
    if !existing_doc.is_empty() {
        parts.push("\n## 现有文档".to_string());
        parts.push("以下是已有的文档内容，请在此基础上完善和补充：".to_string());
        parts.push(format!("\n{}", existing_doc));
    }

    // 保存的响应示例（使用 MD 文档内容）
    if !saved_responses.is_empty() {
        parts.push("\n## 保存的响应文档".to_string());
        parts.push("以下是已保存的响应文档内容，可用于参考接口实际响应情况：".to_string());
        for (i, resp) in saved_responses.iter().take(3).enumerate() {
            if !resp.doc_content.is_empty() {
                parts.push(format!("\n### 示例 {}: {}", i + 1, resp.name));
                let content = &resp.doc_content;
                let truncated = if content.len() > 3000 {
                    format!(
                        "{}...\n\n**注意**：文档内容较长，已截断显示。",
                        truncate_at_char_boundary(content, 3000)
                    )
                } else {
                    content.clone()
                };
                parts.push(format!("\n{}", truncated));
            }
        }
    }

    // 历史请求记录
    if !history_entries.is_empty() {
        parts.push("\n## 历史请求记录".to_string());
        parts.push("以下是历史请求记录，可帮助理解接口实际使用情况：".to_string());
        for (i, entry) in history_entries.iter().take(5).enumerate() {
            parts.push(format!("\n### 请求 {}", i + 1));
            parts.push(format!("- 时间：{}", entry.created_at));
            parts.push(format!("- 状态码：{} {}", entry.status, entry.status_text));
            parts.push(format!("- 响应时间：{}ms", entry.time));
            let body = &entry.response_body;
            let truncated = if body.len() > 2000 {
                format!(
                    "{}...\n\n**注意**：响应体已截断，请根据可见部分分析响应结构。",
                    truncate_at_char_boundary(body, 2000)
                )
            } else {
                body.clone()
            };
            parts.push("- 响应体片段：".to_string());
            parts.push(format!("\n{}", truncated));
        }
    }

    // 输出要求
    parts.push("\n## 输出要求".to_string());
    parts.push("请生成一份完整的接口文档，包括：".to_string());
    parts.push("1. 接口概述（**必须包含请求PATH**）".to_string());
    parts.push("2. 请求参数说明".to_string());
    parts.push("3. 请求示例（**PATH必须正确**）".to_string());
    parts.push("4. **响应参数说明**（**必须完整**）：".to_string());
    parts.push("   - 列出响应 JSON 中的所有字段".to_string());
    parts.push("   - 说明每个字段的类型（string/number/boolean/object/array/null）".to_string());
    parts.push("   - 说明每个字段的含义和用途".to_string());
    parts.push("   - 标注是否为必返回字段".to_string());
    parts.push("   - 对于嵌套对象，逐层展开说明".to_string());
    parts.push("   - 对于数组类型，说明数组元素的类型和结构".to_string());
    parts.push("5. **响应示例**（**必须完整**）：".to_string());
    parts.push("   - 提供至少一个完整的 JSON 响应示例".to_string());
    parts.push("   - 示例应包含所有重要字段".to_string());
    parts.push("   - 使用真实的响应数据作为参考".to_string());
    parts.push("6. 错误码说明（如果有）".to_string());
    parts.push("7. 使用注意事项".to_string());
    parts.push("\n**特别提醒**：".to_string());
    parts.push("- 文档中所有请求示例的PATH必须与上述请求路径保持一致。".to_string());
    parts.push("- **不需要显示完整的请求URL**，只显示请求PATH即可。".to_string());
    parts.push("- **响应参数说明必须覆盖响应 JSON 中的所有可见字段**，不可遗漏。".to_string());
    parts.push("- **响应示例必须足够完整**，展示接口的完整返回结构。".to_string());
    parts.push("\n请直接输出Markdown格式的文档内容，无需其他说明。".to_string());

    parts.join("\n")
}

fn truncate_at_char_boundary(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}
