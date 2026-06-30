use crate::domain::models::common::generate_id;
use crate::domain::models::import::{OpenApi3, Schema};
use crate::domain::models::{Collection, FormField, Header, Param};
use std::collections::HashMap;

pub fn convert_to_collection(
    openapi: OpenApi3,
    target_collection_id: Option<String>,
    root_name: Option<String>,
) -> Result<Collection, String> {
    let root_id = target_collection_id.unwrap_or_else(|| generate_id("col"));
    let root_name = root_name.unwrap_or_else(|| openapi.info.title.clone());

    // 构建 schemas 引用解析上下文
    let empty_schemas = HashMap::new();
    let schemas_ctx: &HashMap<String, Schema> = match &openapi.components {
        Some(c) => &c.schemas,
        None => &empty_schemas,
    };

    let mut tag_map: HashMap<String, Collection> = HashMap::new();
    let mut untagged_apis: Vec<Collection> = Vec::new();

    for tag in &openapi.tags {
        let tag_collection = Collection {
            id: generate_id("col"),
            name: tag.name.clone(),
            description: tag.description.clone(),
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
        tag_map.insert(tag.name.clone(), tag_collection);
    }

    for (path, path_item) in &openapi.paths {
        for (method, operation) in path_item.operations() {
            let api = convert_operation_to_api(path, method, operation, schemas_ctx);

            if operation.tags.is_empty() {
                untagged_apis.push(api);
            } else if let Some(tag_name) = operation.tags.first() {
                if !tag_map.contains_key(tag_name) {
                    let tag_collection = Collection {
                        id: generate_id("col"),
                        name: tag_name.clone(),
                        description: None,
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
                    tag_map.insert(tag_name.clone(), tag_collection);
                }
                if let Some(tag_collection) = tag_map.get_mut(tag_name) {
                    tag_collection.children.push(api);
                }
            }
        }
    }

    let mut children: Vec<Collection> = tag_map.into_values().collect();
    children.sort_by(|a, b| a.name.cmp(&b.name));

    if !untagged_apis.is_empty() {
        let untagged_collection = Collection {
            id: generate_id("col"),
            name: "未分类".to_string(),
            description: Some("未分配标签的接口".to_string()),
            item_type: "collection".to_string(),
            children: untagged_apis,
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
        children.push(untagged_collection);
    }

    let root = Collection {
        id: root_id,
        name: root_name,
        description: openapi.info.description.clone(),
        item_type: "collection".to_string(),
        children,
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

    Ok(root)
}

fn convert_operation_to_api(
    path: &str,
    method: &str,
    operation: &crate::domain::models::import::Operation,
    schemas_ctx: &HashMap<String, Schema>,
) -> Collection {
    let api_id = generate_id("api");

    let name = operation
        .summary
        .clone()
        .or(operation.operation_id.clone())
        .unwrap_or_else(|| format!("{} {}", method, path));

    let name = if name.chars().count() > 50 {
        name.chars().take(47).collect::<String>() + "..."
    } else {
        name
    };

    let mut headers: Vec<Header> = Vec::new();
    let mut params: Vec<Param> = Vec::new();

    for param in &operation.parameters {
        match param.location.as_str() {
            "query" => {
                let value = param
                    .example
                    .clone()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_default();
                params.push(Param {
                    key: param.name.clone(),
                    value,
                    enabled: true,
                    description: param.description.clone(),
                });
            }
            "header" => {
                let value = param
                    .example
                    .clone()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| {
                        param
                            .schema
                            .as_ref()
                            .and_then(|s| {
                                s.example
                                    .clone()
                                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                            })
                            .unwrap_or_default()
                    });
                headers.push(Header {
                    key: param.name.clone(),
                    value,
                    enabled: true,
                    description: param.description.clone(),
                });
            }
            "path" => {}
            _ => {}
        }
    }

    let (body, body_type, form_fields, content_type) = extract_request_body(operation, schemas_ctx);

    if let Some(ct) = content_type {
        headers.push(Header {
            key: "Content-Type".to_string(),
            value: ct,
            enabled: true,
            description: None,
        });
    }

    Collection {
        id: api_id,
        name,
        description: operation.description.clone(),
        item_type: "api".to_string(),
        children: Vec::new(),
        method: Some(method.to_string()),
        url: Some(path.to_string()),
        params: if params.is_empty() {
            None
        } else {
            Some(params)
        },
        headers: if headers.is_empty() {
            None
        } else {
            Some(headers)
        },
        body,
        body_type,
        form_fields,
        saved_responses: None,
        common_headers: None,
        collection_variables: None,
        ws_config: None,
    }
}

fn extract_request_body(
    operation: &crate::domain::models::import::Operation,
    schemas_ctx: &HashMap<String, Schema>,
) -> (
    Option<String>,
    Option<String>,
    Option<Vec<FormField>>,
    Option<String>,
) {
    let request_body = match &operation.request_body {
        Some(rb) => rb,
        None => return (None, None, None, None),
    };

    for (content_type, media_type) in &request_body.content {
        match content_type.as_str() {
            "application/json" => {
                let schema = media_type.schema.as_ref();
                if let Some(s) = schema {
                    let resolved = resolve_schema_ref(s, schemas_ctx);
                    if has_binary_field(&resolved, schemas_ctx) {
                        let form_fields = extract_form_fields(media_type, schemas_ctx);
                        return (
                            None,
                            Some("form-data".to_string()),
                            form_fields,
                            Some("multipart/form-data".to_string()),
                        );
                    }
                }
                let body = extract_json_example(media_type, schemas_ctx);
                return (
                    body,
                    Some("raw".to_string()),
                    None,
                    Some(content_type.clone()),
                );
            }
            "application/x-www-form-urlencoded" => {
                let form_fields = extract_form_fields(media_type, schemas_ctx);
                return (
                    None,
                    Some("form-data".to_string()),
                    form_fields,
                    Some(content_type.clone()),
                );
            }
            "multipart/form-data" => {
                let form_fields = extract_form_fields(media_type, schemas_ctx);
                return (
                    None,
                    Some("form-data".to_string()),
                    form_fields,
                    Some(content_type.clone()),
                );
            }
            "text/plain" => {
                let body = media_type
                    .example
                    .clone()
                    .and_then(|v| v.as_str().map(|s| s.to_string()));
                return (
                    body,
                    Some("raw".to_string()),
                    None,
                    Some(content_type.clone()),
                );
            }
            _ => {}
        }
    }

    (None, None, None, None)
}

fn has_binary_field(schema: &Schema, schemas_ctx: &HashMap<String, Schema>) -> bool {
    for prop_schema in &schema.properties {
        let resolved = resolve_schema_ref(prop_schema.1, schemas_ctx);
        if resolved
            .format
            .as_ref()
            .map(|f| f == "binary" || f == "byte")
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

fn extract_json_example(
    media_type: &crate::domain::models::import::MediaType,
    schemas_ctx: &HashMap<String, Schema>,
) -> Option<String> {
    if let Some(ref example) = media_type.example {
        return Some(serde_json::to_string_pretty(example).unwrap_or_default());
    }

    if let Some(ref examples) = media_type.examples {
        if let Some(first_example) = examples.values().next() {
            if let Some(ref value) = first_example.value {
                return Some(serde_json::to_string_pretty(value).unwrap_or_default());
            }
            if let Some(ref external_value) = first_example.external_value {
                return Some(format!("[External Example: {}]", external_value));
            }
        }
    }

    if let Some(ref schema) = media_type.schema {
        if let Some(ref example) = schema.example {
            return Some(serde_json::to_string_pretty(example).unwrap_or_default());
        }

        let generated = generate_example_from_schema(schema, schemas_ctx);
        if !generated.is_null() {
            return Some(serde_json::to_string_pretty(&generated).unwrap_or_default());
        }
    }

    None
}

fn generate_example_from_schema(
    schema: &Schema,
    schemas_ctx: &HashMap<String, Schema>,
) -> serde_json::Value {
    if let Some(ref example) = schema.example {
        return example.clone();
    }

    if let Some(ref ref_path) = schema.ref_path {
        if ref_path.starts_with("#/components/schemas/") {
            let schema_name = ref_path.strip_prefix("#/components/schemas/").unwrap();
            if let Some(ref_schema) = schemas_ctx.get(schema_name) {
                return generate_example_from_schema(ref_schema, schemas_ctx);
            }
        }
    }

    match schema.schema_type.as_deref() {
        Some("string") => {
            if let Some(ref format) = schema.format {
                match format.as_str() {
                    "date" => serde_json::Value::String("2024-01-01".to_string()),
                    "date-time" => serde_json::Value::String("2024-01-01T00:00:00Z".to_string()),
                    "email" => serde_json::Value::String("user@example.com".to_string()),
                    "uri" => serde_json::Value::String("https://example.com".to_string()),
                    "uuid" => serde_json::Value::String(
                        "00000000-0000-0000-0000-000000000000".to_string(),
                    ),
                    _ => serde_json::Value::String("".to_string()),
                }
            } else if !schema.enum_values.is_empty() {
                serde_json::Value::String(schema.enum_values[0].clone())
            } else {
                serde_json::Value::String("".to_string())
            }
        }
        Some("integer") => serde_json::Value::Number(serde_json::Number::from(0)),
        Some("number") => serde_json::Value::Number(
            serde_json::Number::from_f64(0.0).unwrap_or_else(|| serde_json::Number::from(0)),
        ),
        Some("boolean") => serde_json::Value::Bool(false),
        Some("array") => {
            if let Some(ref items) = schema.items {
                serde_json::Value::Array(vec![generate_example_from_schema(items, schemas_ctx)])
            } else {
                serde_json::Value::Array(Vec::new())
            }
        }
        Some("object") => {
            let mut obj = serde_json::Map::new();
            for (key, prop_schema) in &schema.properties {
                obj.insert(
                    key.clone(),
                    generate_example_from_schema(prop_schema, schemas_ctx),
                );
            }
            serde_json::Value::Object(obj)
        }
        _ => serde_json::Value::Null,
    }
}

fn extract_form_fields(
    media_type: &crate::domain::models::import::MediaType,
    schemas_ctx: &HashMap<String, Schema>,
) -> Option<Vec<FormField>> {
    let schema = media_type.schema.as_ref()?;

    let resolved_schema = resolve_schema_ref(schema, schemas_ctx);

    let mut fields = Vec::new();
    for (name, prop_schema) in &resolved_schema.properties {
        let resolved_prop = resolve_schema_ref(prop_schema, schemas_ctx);

        let is_file = resolved_prop
            .format
            .as_ref()
            .map(|f| f == "binary" || f == "byte")
            .unwrap_or(false);

        let field_type = if is_file { "file" } else { "text" };

        let value = if is_file {
            String::new()
        } else if let Some(ref example) = resolved_prop.example {
            example.as_str().unwrap_or("").to_string()
        } else {
            generate_example_from_schema(&resolved_prop, schemas_ctx)
                .as_str()
                .unwrap_or("")
                .to_string()
        };

        fields.push(FormField {
            key: name.clone(),
            value,
            field_type: field_type.to_string(),
            enabled: true,
            files: None,
        });
    }

    if fields.is_empty() {
        None
    } else {
        Some(fields)
    }
}

fn resolve_schema_ref(schema: &Schema, schemas_ctx: &HashMap<String, Schema>) -> Schema {
    if let Some(ref ref_path) = schema.ref_path {
        if ref_path.starts_with("#/components/schemas/") {
            let schema_name = ref_path.strip_prefix("#/components/schemas/").unwrap();
            if let Some(ref_schema) = schemas_ctx.get(schema_name) {
                return resolve_schema_ref(ref_schema, schemas_ctx);
            }
        }
    }
    schema.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::import::MediaType;
    use std::collections::HashMap;

    #[test]
    fn test_real_openapi_login_request() {
        let openapi_json = r##"
        {
          "openapi": "3.0.1",
          "info": {
            "title": "Test API",
            "version": "v0"
          },
          "paths": {
            "/auth/login": {
              "post": {
                "tags": ["用户认证"],
                "summary": "用户登录",
                "parameters": [
                  {
                    "name": "X-Auth-Token",
                    "in": "header",
                    "description": "认证Token",
                    "required": false,
                    "schema": {
                      "type": "string"
                    },
                    "example": "test-token-123"
                  }
                ],
                "requestBody": {
                  "content": {
                    "application/json": {
                      "schema": {
                        "$ref": "#/components/schemas/LoginRequest"
                      }
                    }
                  },
                  "required": true
                },
                "responses": {
                  "200": {
                    "description": "OK"
                  }
                }
              }
            }
          },
          "components": {
            "schemas": {
              "LoginRequest": {
                "type": "object",
                "properties": {
                  "account": {
                    "type": "string",
                    "description": "账号",
                    "example": "admin"
                  },
                  "phone": {
                    "type": "string",
                    "description": "手机号",
                    "example": "13800138000"
                  },
                  "password": {
                    "type": "string",
                    "description": "密码",
                    "example": "123456"
                  },
                  "loginType": {
                    "type": "string",
                    "description": "登录方式",
                    "enum": ["ACCOUNT_PASSWORD", "PHONE_PASSWORD"]
                  }
                },
                "description": "用户登录请求"
              }
            }
          }
        }
        "##;

        use crate::domain::services::import::openapi_parser_domain::parse_openapi;
        let openapi = parse_openapi(openapi_json.trim(), "json").unwrap();

        let empty_schemas = HashMap::new();
        let schemas_ctx: &HashMap<String, Schema> = match &openapi.components {
            Some(c) => &c.schemas,
            None => &empty_schemas,
        };

        let login_path = openapi.paths.get("/auth/login").unwrap();
        let login_op = login_path.post.as_ref().unwrap();

        let request_body = login_op.request_body.as_ref().unwrap();
        let media_type = request_body.content.get("application/json").unwrap();

        let result = extract_json_example(media_type, schemas_ctx);

        assert!(result.is_some());
        let body_str = result.unwrap();
        println!("Generated body: {}", body_str);

        let body: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert!(body.is_object());

        let obj = body.as_object().unwrap();
        assert_eq!(obj.get("account").unwrap().as_str().unwrap(), "admin");
        assert_eq!(obj.get("phone").unwrap().as_str().unwrap(), "13800138000");
        assert_eq!(obj.get("password").unwrap().as_str().unwrap(), "123456");
        assert_eq!(
            obj.get("loginType").unwrap().as_str().unwrap(),
            "ACCOUNT_PASSWORD"
        );

        let collection = convert_to_collection(openapi, None, None).unwrap();
        let auth_collection = collection
            .children
            .iter()
            .find(|c| c.name == "用户认证")
            .unwrap();
        let login_api = auth_collection
            .children
            .iter()
            .find(|c| c.item_type == "api")
            .unwrap();

        assert!(login_api.headers.is_some());
        let headers = login_api.headers.as_ref().unwrap();
        assert_eq!(headers.len(), 2);

        let auth_token_header = headers.iter().find(|h| h.key == "X-Auth-Token").unwrap();
        assert_eq!(auth_token_header.value, "test-token-123");

        let content_type_header = headers.iter().find(|h| h.key == "Content-Type").unwrap();
        assert_eq!(content_type_header.value, "application/json");
    }

    #[test]
    fn test_generate_example_from_schema_with_property_example() {
        let mut schemas_ctx = HashMap::new();

        let login_request_schema = Schema {
            schema_type: Some("object".to_string()),
            format: None,
            description: Some("用户登录请求".to_string()),
            properties: {
                let mut props = HashMap::new();
                props.insert(
                    "account".to_string(),
                    Schema {
                        schema_type: Some("string".to_string()),
                        example: Some(serde_json::Value::String("admin".to_string())),
                        format: None,
                        description: Some("账号".to_string()),
                        properties: HashMap::new(),
                        items: None,
                        required: vec![],
                        enum_values: vec![],
                        default: None,
                        ref_path: None,
                    },
                );
                props.insert(
                    "password".to_string(),
                    Schema {
                        schema_type: Some("string".to_string()),
                        example: Some(serde_json::Value::String("123456".to_string())),
                        format: None,
                        description: Some("密码".to_string()),
                        properties: HashMap::new(),
                        items: None,
                        required: vec![],
                        enum_values: vec![],
                        default: None,
                        ref_path: None,
                    },
                );
                props.insert(
                    "loginType".to_string(),
                    Schema {
                        schema_type: Some("string".to_string()),
                        example: None,
                        format: None,
                        description: Some("登录方式".to_string()),
                        properties: HashMap::new(),
                        items: None,
                        required: vec![],
                        enum_values: vec![
                            "ACCOUNT_PASSWORD".to_string(),
                            "PHONE_PASSWORD".to_string(),
                        ],
                        default: None,
                        ref_path: None,
                    },
                );
                props
            },
            items: None,
            required: vec!["account".to_string(), "password".to_string()],
            enum_values: vec![],
            default: None,
            example: None,
            ref_path: None,
        };

        schemas_ctx.insert("LoginRequest".to_string(), login_request_schema.clone());

        let ref_schema = Schema {
            schema_type: None,
            ref_path: Some("#/components/schemas/LoginRequest".to_string()),
            format: None,
            description: None,
            properties: HashMap::new(),
            items: None,
            required: vec![],
            enum_values: vec![],
            default: None,
            example: None,
        };

        let result = generate_example_from_schema(&ref_schema, &schemas_ctx);

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("account").unwrap().as_str().unwrap(), "admin");
        assert_eq!(obj.get("password").unwrap().as_str().unwrap(), "123456");
        assert_eq!(
            obj.get("loginType").unwrap().as_str().unwrap(),
            "ACCOUNT_PASSWORD"
        );
    }

    #[test]
    fn test_extract_json_example_with_ref_schema() {
        let mut schemas_ctx = HashMap::new();

        let login_request_schema = Schema {
            schema_type: Some("object".to_string()),
            format: None,
            description: Some("用户登录请求".to_string()),
            properties: {
                let mut props = HashMap::new();
                props.insert(
                    "account".to_string(),
                    Schema {
                        schema_type: Some("string".to_string()),
                        example: Some(serde_json::Value::String("admin".to_string())),
                        format: None,
                        description: Some("账号".to_string()),
                        properties: HashMap::new(),
                        items: None,
                        required: vec![],
                        enum_values: vec![],
                        default: None,
                        ref_path: None,
                    },
                );
                props
            },
            items: None,
            required: vec![],
            enum_values: vec![],
            default: None,
            example: None,
            ref_path: None,
        };

        schemas_ctx.insert("LoginRequest".to_string(), login_request_schema);

        let media_type = MediaType {
            schema: Some(Schema {
                ref_path: Some("#/components/schemas/LoginRequest".to_string()),
                schema_type: None,
                format: None,
                description: None,
                properties: HashMap::new(),
                items: None,
                required: vec![],
                enum_values: vec![],
                default: None,
                example: None,
            }),
            example: None,
            examples: None,
        };

        let result = extract_json_example(&media_type, &schemas_ctx);

        assert!(result.is_some());
        let body_str = result.unwrap();
        let body: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert!(body.is_object());
        assert_eq!(body.get("account").unwrap().as_str().unwrap(), "admin");
    }
}
