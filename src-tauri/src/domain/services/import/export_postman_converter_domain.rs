use crate::domain::models::common::generate_id;
use crate::domain::models::import::{
    PostmanBody, PostmanBodyOptions, PostmanCollection, PostmanFormDataParam, PostmanHeader,
    PostmanInfo, PostmanItem, PostmanItemGroup, PostmanItemOrGroup, PostmanRawOptions,
    PostmanRequest, PostmanRequestObject, PostmanUrl, PostmanUrlEncodedParam, PostmanUrlObject,
    PostmanVariable,
};
use crate::domain::models::{Collection, FormField, Header, Variable};

const POSTMAN_SCHEMA: &str = "https://schema.getpostman.com/json/collection/v2.1.0/collection.json";

pub fn convert_collection_to_postman(collection: &Collection) -> PostmanCollection {
    let info = PostmanInfo {
        postman_id: Some(generate_id("pm")),
        name: collection.name.clone(),
        description: collection.description.clone(),
        schema: POSTMAN_SCHEMA.to_string(),
    };

    let variable = convert_variables_to_postman(&collection.collection_variables);

    let item = convert_items_to_postman(&collection.children);

    PostmanCollection {
        info,
        item,
        variable,
        auth: None,
        event: Vec::new(),
    }
}

fn convert_items_to_postman(items: &[Collection]) -> Vec<PostmanItemOrGroup> {
    items
        .iter()
        .map(|item| {
            if item.item_type == "api" {
                PostmanItemOrGroup::Item(convert_api_to_postman_item(item))
            } else {
                PostmanItemOrGroup::Group(convert_collection_to_postman_group(item))
            }
        })
        .collect()
}

fn convert_collection_to_postman_group(collection: &Collection) -> PostmanItemGroup {
    PostmanItemGroup {
        name: Some(collection.name.clone()),
        description: collection.description.clone(),
        item: convert_items_to_postman(&collection.children),
        variable: convert_variables_to_postman(&collection.collection_variables),
        auth: None,
        event: Vec::new(),
    }
}

fn convert_api_to_postman_item(api: &Collection) -> PostmanItem {
    let request = PostmanRequest::Object(PostmanRequestObject {
        method: api.method.clone().unwrap_or_else(|| "GET".to_string()),
        url: Some(convert_url_to_postman(&api.url)),
        header: convert_headers_to_postman(&api.headers, &api.common_headers),
        body: convert_body_to_postman(&api.body, &api.body_type, &api.form_fields),
        auth: None,
        description: api.description.clone(),
    });

    PostmanItem {
        name: Some(api.name.clone()),
        description: api.description.clone(),
        request,
        response: Vec::new(),
        variable: convert_variables_to_postman(&api.collection_variables),
        event: Vec::new(),
        auth: None,
    }
}

fn convert_url_to_postman(url: &Option<String>) -> PostmanUrl {
    match url {
        None => PostmanUrl::String(String::new()),
        Some(url_str) => {
            if url_str.is_empty() {
                return PostmanUrl::String(String::new());
            }

            // 尝试解析 URL 为结构化对象
            if let Ok(parsed) = url::Url::parse(url_str) {
                let protocol = parsed.scheme().to_string();

                let host_str = parsed.host_str().unwrap_or("");
                let host = if host_str.contains('.') {
                    PostmanHost::Array(host_str.split('.').map(|s| s.to_string()).collect())
                } else {
                    PostmanHost::String(host_str.to_string())
                };

                let port = parsed.port().map(|p| p.to_string());

                let path_str = parsed.path();
                let path = if path_str.is_empty() || path_str == "/" {
                    PostmanPath::Array(Vec::new())
                } else {
                    let segments: Vec<String> = path_str
                        .split('/')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect();
                    PostmanPath::Array(segments)
                };

                let query: Vec<PostmanQueryParam> = parsed
                    .query_pairs()
                    .map(|(k, v)| PostmanQueryParam {
                        key: Some(k.to_string()),
                        value: Some(v.to_string()),
                        disabled: false,
                    })
                    .collect();

                PostmanUrl::Object(PostmanUrlObject {
                    raw: Some(url_str.clone()),
                    protocol: Some(protocol),
                    host: Some(host),
                    path: Some(path),
                    port,
                    query,
                })
            } else {
                PostmanUrl::String(url_str.clone())
            }
        }
    }
}

use crate::domain::models::import::{PostmanHost, PostmanPath, PostmanQueryParam};

fn convert_headers_to_postman(
    headers: &Option<Vec<Header>>,
    common_headers: &Option<Vec<Header>>,
) -> Vec<PostmanHeader> {
    let mut result = Vec::new();

    // 先添加通用请求头
    if let Some(common) = common_headers {
        for h in common {
            if h.enabled && !h.key.trim().is_empty() {
                result.push(PostmanHeader {
                    key: h.key.clone(),
                    value: Some(h.value.clone()),
                    disabled: false,
                });
            }
        }
    }

    // 再添加接口请求头
    if let Some(hdrs) = headers {
        for h in hdrs {
            if h.enabled && !h.key.trim().is_empty() {
                // 检查是否已存在（避免重复）
                if !result.iter().any(|ph| ph.key == h.key) {
                    result.push(PostmanHeader {
                        key: h.key.clone(),
                        value: Some(h.value.clone()),
                        disabled: false,
                    });
                }
            }
        }
    }

    result
}

fn convert_body_to_postman(
    body: &Option<String>,
    body_type: &Option<String>,
    form_fields: &Option<Vec<FormField>>,
) -> Option<PostmanBody> {
    let actual_type = body_type.clone().unwrap_or_else(|| "raw".to_string());

    match actual_type.as_str() {
        "raw" => {
            if let Some(content) = body {
                if !content.is_empty() {
                    // 根据内容推断语言类型
                    let language = infer_language(content);

                    Some(PostmanBody {
                        mode: Some("raw".to_string()),
                        raw: Some(content.clone()),
                        urlencoded: Vec::new(),
                        formdata: Vec::new(),
                        options: Some(PostmanBodyOptions {
                            raw: Some(PostmanRawOptions {
                                language: Some(language),
                            }),
                        }),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
        "form-data" => {
            if let Some(fields) = form_fields {
                let formdata: Vec<PostmanFormDataParam> = fields
                    .iter()
                    .filter(|f| f.enabled && !f.key.is_empty())
                    .map(|f| match f.field_type.as_str() {
                        "file" => PostmanFormDataParam::File {
                            key: f.key.clone(),
                            src: f
                                .files
                                .as_ref()
                                .and_then(|files| files.first())
                                .map(|fi| fi.path.clone()),
                            disabled: false,
                            param_type: Some("file".to_string()),
                        },
                        _ => PostmanFormDataParam::Text {
                            key: f.key.clone(),
                            value: Some(f.value.clone()),
                            disabled: false,
                            param_type: Some("text".to_string()),
                        },
                    })
                    .collect();

                if !formdata.is_empty() {
                    Some(PostmanBody {
                        mode: Some("formdata".to_string()),
                        raw: None,
                        urlencoded: Vec::new(),
                        formdata,
                        options: None,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
        "x-www-form-urlencoded" => {
            if let Some(fields) = form_fields {
                let urlencoded: Vec<PostmanUrlEncodedParam> = fields
                    .iter()
                    .filter(|f| f.enabled && !f.key.is_empty())
                    .map(|f| PostmanUrlEncodedParam {
                        key: Some(f.key.clone()),
                        value: Some(f.value.clone()),
                        disabled: false,
                    })
                    .collect();

                if !urlencoded.is_empty() {
                    Some(PostmanBody {
                        mode: Some("urlencoded".to_string()),
                        raw: None,
                        urlencoded,
                        formdata: Vec::new(),
                        options: None,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
        "binary" => {
            // Postman 的 file 模式不支持保存文件路径，需要用户重新选择
            Some(PostmanBody {
                mode: Some("file".to_string()),
                raw: None,
                urlencoded: Vec::new(),
                formdata: Vec::new(),
                options: None,
            })
        }
        _ => None,
    }
}

fn infer_language(content: &str) -> String {
    // 尝试解析为 JSON
    if serde_json::from_str::<serde_json::Value>(content).is_ok() {
        return "json".to_string();
    }

    // 检查是否像 JSON（包含花括号或方括号）
    let trimmed = content.trim();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return "json".to_string();
    }

    // 检查是否像 XML
    if trimmed.starts_with('<') {
        return "xml".to_string();
    }

    // 检查是否像 HTML
    if trimmed.starts_with("<!DOCTYPE") || trimmed.starts_with("<html") {
        return "html".to_string();
    }

    // 默认为 text
    "text".to_string()
}

fn convert_variables_to_postman(vars: &Option<Vec<Variable>>) -> Vec<PostmanVariable> {
    match vars {
        None => Vec::new(),
        Some(variables) => variables
            .iter()
            .filter(|v| v.enabled && !v.key.is_empty())
            .map(|v| PostmanVariable {
                id: Some(generate_id("pmvar")),
                key: Some(v.key.clone()),
                value: Some(v.value.clone()),
                var_type: Some("string".to_string()),
                description: v.description.clone(),
            })
            .collect(),
    }
}
