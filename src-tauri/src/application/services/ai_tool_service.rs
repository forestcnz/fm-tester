//! AI 工具执行服务
//!
//! 实现 @fm 工作区上下文聊天所需的工具：
//! - list_workspace_apis：列出工作区全部接口
//! - get_api_detail：获取单个接口完整定义
//! - get_api_doc：获取接口 Markdown 文档
//! - search_apis：关键词搜索接口
//!
//! 工具返回 JSON 字符串（完整内容，不做截断）。

use crate::application::services::{
    read_collections, CollectionApplicationService, HistoryApplicationService,
    MdApplicationService, ResponseApplicationService,
};
use crate::domain::models::Collection;

pub struct AiToolService;

impl AiToolService {
    /// 执行工具调用，返回 JSON 字符串结果
    pub fn execute(workspace_id: &str, name: &str, arguments: &str) -> String {
        match name {
            "list_workspace_apis" => Self::list_workspace_apis(workspace_id),
            "get_api_detail" => Self::get_api_detail(workspace_id, arguments),
            "get_api_doc" => Self::get_api_doc(workspace_id, arguments),
            "get_api_responses" => Self::get_api_responses(workspace_id, arguments),
            "get_api_history" => Self::get_api_history(workspace_id, arguments),
            "search_apis" => Self::search_apis(workspace_id, arguments),
            other => format!("{{\"error\":\"未知工具: {}\"}}", escape(other)),
        }
    }

    /// 列出工作区全部接口（id/名称/方法/URL/描述/集合路径）
    fn list_workspace_apis(workspace_id: &str) -> String {
        let config = match read_collections(workspace_id) {
            Ok(c) => c,
            Err(e) => return format!("{{\"error\":\"读取集合失败: {}\"}}", escape(&e)),
        };

        let mut apis: Vec<serde_json::Value> = Vec::new();
        for root in &config.collections {
            Self::collect_apis(root, "", &mut apis);
        }

        serde_json::json!({ "apis": apis }).to_string()
    }

    /// 递归收集 API 节点（path 为该节点所属的集合路径）
    fn collect_apis(node: &Collection, path: &str, apis: &mut Vec<serde_json::Value>) {
        if node.is_api() {
            apis.push(serde_json::json!({
                "id": node.id,
                "name": node.name,
                "method": node.method.clone().unwrap_or_default(),
                "url": node.url.clone().unwrap_or_default(),
                "description": node.description.clone().unwrap_or_default(),
                "collection": path,
            }));
            return;
        }
        let path_for_children = if path.is_empty() {
            node.name.clone()
        } else {
            format!("{}/{}", path, node.name)
        };
        for child in &node.children {
            Self::collect_apis(child, &path_for_children, apis);
        }
    }

    /// 获取单个接口完整定义
    fn get_api_detail(workspace_id: &str, arguments: &str) -> String {
        let api_id = match parse_string_arg(arguments, "api_id") {
            Some(id) => id,
            None => return "{\"error\":\"缺少参数 api_id\"}".to_string(),
        };

        let config = match read_collections(workspace_id) {
            Ok(c) => c,
            Err(e) => return format!("{{\"error\":\"读取集合失败: {}\"}}", escape(&e)),
        };

        match CollectionApplicationService::find_api(&config, &api_id) {
            Some(api) => Self::api_detail_json(&api).to_string(),
            None => format!("{{\"error\":\"接口不存在: {}\"}}", escape(&api_id)),
        }
    }

    /// 获取接口 Markdown 文档
    fn get_api_doc(workspace_id: &str, arguments: &str) -> String {
        let api_id = match parse_string_arg(arguments, "api_id") {
            Some(id) => id,
            None => return "{\"error\":\"缺少参数 api_id\"}".to_string(),
        };

        match MdApplicationService::get_doc(workspace_id, &api_id) {
            Ok(doc) if !doc.is_empty() => serde_json::json!({ "doc": doc }).to_string(),
            _ => "{\"doc\":\"\",\"hint\":\"该接口暂无文档\"}".to_string(),
        }
    }

    /// 获取接口已保存的响应示例
    fn get_api_responses(workspace_id: &str, arguments: &str) -> String {
        let api_id = match parse_string_arg(arguments, "api_id") {
            Some(id) => id,
            None => return "{\"error\":\"缺少参数 api_id\"}".to_string(),
        };
        let entries = match ResponseApplicationService::get_by_api(workspace_id, &api_id) {
            Ok(e) => e,
            Err(e) => return format!("{{\"error\":\"读取保存响应失败: {}\"}}", escape(&e)),
        };
        let mut responses: Vec<serde_json::Value> = Vec::new();
        for entry in entries {
            if let Ok(Some(resp)) = ResponseApplicationService::get(workspace_id, &entry.id) {
                responses.push(serde_json::json!({
                    "id": resp.id,
                    "name": resp.name,
                    "created_at": resp.created_at,
                    "doc_content": resp.doc_content,
                }));
            }
        }
        serde_json::json!({ "api_id": api_id, "responses": responses }).to_string()
    }

    /// 获取接口请求历史（按 api_id 直接查询，最近 10 条）
    fn get_api_history(workspace_id: &str, arguments: &str) -> String {
        let api_id = match parse_string_arg(arguments, "api_id") {
            Some(id) => id,
            None => return "{\"error\":\"缺少参数 api_id\"}".to_string(),
        };
        let entries = match HistoryApplicationService::get_by_api(workspace_id, &api_id, 10) {
            Ok(e) => e,
            Err(e) => return format!("{{\"error\":\"读取历史失败: {}\"}}", escape(&e)),
        };
        let history: Vec<serde_json::Value> = entries
            .into_iter()
            .map(|e| {
                serde_json::json!({
                    "created_at": e.created_at,
                    "method": e.method,
                    "url": e.url,
                    "status": e.status,
                    "status_text": e.status_text,
                    "time_ms": e.time,
                    "size": e.size,
                    "response_body": e.response_body,
                })
            })
            .collect();
        serde_json::json!({ "api_id": api_id, "history": history }).to_string()
    }

    /// 按关键词搜索接口
    fn search_apis(workspace_id: &str, arguments: &str) -> String {
        let keyword = match parse_string_arg(arguments, "keyword") {
            Some(k) => k,
            None => return "{\"error\":\"缺少参数 keyword\"}".to_string(),
        };

        let config = match read_collections(workspace_id) {
            Ok(c) => c,
            Err(e) => return format!("{{\"error\":\"读取集合失败: {}\"}}", escape(&e)),
        };

        let mut all: Vec<serde_json::Value> = Vec::new();
        for root in &config.collections {
            Self::collect_apis(root, "", &mut all);
        }

        let kw = keyword.to_lowercase();
        let matched: Vec<serde_json::Value> = all
            .into_iter()
            .filter(|a| {
                let name = a["name"].as_str().unwrap_or("").to_lowercase();
                let url = a["url"].as_str().unwrap_or("").to_lowercase();
                let desc = a["description"].as_str().unwrap_or("").to_lowercase();
                name.contains(&kw) || url.contains(&kw) || desc.contains(&kw)
            })
            .collect();

        serde_json::json!({
            "keyword": keyword,
            "matched": matched.len(),
            "apis": matched,
        })
        .to_string()
    }

    /// 构造接口完整信息 JSON（不含集合路径）
    fn api_detail_json(api: &Collection) -> serde_json::Value {
        serde_json::json!({
            "id": api.id,
            "name": api.name,
            "description": api.description.clone().unwrap_or_default(),
            "method": api.method.clone().unwrap_or_default(),
            "url": api.url.clone().unwrap_or_default(),
            "params": api.params.as_ref().map(|ps| {
                ps.iter().filter(|p| p.enabled)
                    .map(|p| serde_json::json!({"key": p.key, "value": p.value}))
                    .collect::<Vec<_>>()
            }).unwrap_or_default(),
            "headers": api.headers.as_ref().map(|hs| {
                hs.iter().filter(|h| h.enabled)
                    .map(|h| serde_json::json!({"key": h.key, "value": h.value}))
                    .collect::<Vec<_>>()
            }).unwrap_or_default(),
            "body_type": api.body_type.clone().unwrap_or_default(),
            "body": api.body.clone().unwrap_or_default(),
            "form_fields": api.form_fields.as_ref().map(|fs| {
                fs.iter().filter(|f| f.enabled)
                    .map(|f| serde_json::json!({"key": f.key, "value": f.value, "type": f.field_type}))
                    .collect::<Vec<_>>()
            }).unwrap_or_default(),
        })
    }
}

/// 从 JSON 参数字符串中取一个字符串字段
fn parse_string_arg(arguments: &str, key: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(arguments)
        .ok()
        .and_then(|v| v.get(key).and_then(|x| x.as_str()).map(|s| s.to_string()))
}

/// 转义字符串中的特殊字符用于 JSON 内嵌
fn escape(s: &str) -> String {
    serde_json::Value::String(s.to_string()).to_string()
}
