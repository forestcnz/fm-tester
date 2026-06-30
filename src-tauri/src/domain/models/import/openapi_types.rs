use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenApi3 {
    pub openapi: String,
    pub info: Info,
    #[serde(default)]
    pub servers: Vec<Server>,
    #[serde(default)]
    pub paths: HashMap<String, PathItem>,
    #[serde(default)]
    pub tags: Vec<Tag>,
    #[serde(default)]
    pub components: Option<Components>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Info {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Server {
    pub url: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tag {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PathItem {
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub get: Option<Operation>,
    #[serde(default)]
    pub post: Option<Operation>,
    #[serde(default)]
    pub put: Option<Operation>,
    #[serde(default)]
    pub delete: Option<Operation>,
    #[serde(default)]
    pub patch: Option<Operation>,
    #[serde(default)]
    pub options: Option<Operation>,
    #[serde(default)]
    pub head: Option<Operation>,
    #[serde(default)]
    pub trace: Option<Operation>,
}

impl PathItem {
    pub fn operations(&self) -> Vec<(&'static str, &Operation)> {
        let mut ops = Vec::new();
        if let Some(ref op) = self.get {
            ops.push(("GET", op));
        }
        if let Some(ref op) = self.post {
            ops.push(("POST", op));
        }
        if let Some(ref op) = self.put {
            ops.push(("PUT", op));
        }
        if let Some(ref op) = self.delete {
            ops.push(("DELETE", op));
        }
        if let Some(ref op) = self.patch {
            ops.push(("PATCH", op));
        }
        if let Some(ref op) = self.options {
            ops.push(("OPTIONS", op));
        }
        if let Some(ref op) = self.head {
            ops.push(("HEAD", op));
        }
        if let Some(ref op) = self.trace {
            ops.push(("TRACE", op));
        }
        ops
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Operation {
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "operationId", default)]
    pub operation_id: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    #[serde(rename = "requestBody", default)]
    pub request_body: Option<RequestBody>,
    #[serde(default)]
    pub responses: HashMap<String, Response>,
    #[serde(default)]
    pub deprecated: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub schema: Option<Schema>,
    #[serde(default)]
    pub example: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestBody {
    #[serde(default)]
    pub description: Option<String>,
    pub content: HashMap<String, MediaType>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MediaType {
    #[serde(default)]
    pub schema: Option<Schema>,
    #[serde(default)]
    pub example: Option<serde_json::Value>,
    #[serde(default)]
    pub examples: Option<HashMap<String, Example>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response {
    pub description: String,
    #[serde(default)]
    pub headers: HashMap<String, OpenApiHeader>,
    #[serde(default)]
    pub content: HashMap<String, MediaType>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenApiHeader {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub schema: Option<Schema>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub deprecated: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Schema {
    #[serde(rename = "type", default)]
    pub schema_type: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub properties: HashMap<String, Schema>,
    #[serde(default)]
    pub items: Option<Box<Schema>>,
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(rename = "enum", default)]
    pub enum_values: Vec<String>,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    #[serde(default)]
    pub example: Option<serde_json::Value>,
    #[serde(rename = "$ref", default)]
    pub ref_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Example {
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub value: Option<serde_json::Value>,
    #[serde(rename = "externalValue", default)]
    pub external_value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Components {
    #[serde(default)]
    pub schemas: HashMap<String, Schema>,
    #[serde(default)]
    pub responses: HashMap<String, Response>,
    #[serde(default)]
    pub parameters: HashMap<String, Parameter>,
    #[serde(default)]
    pub examples: HashMap<String, Example>,
    #[serde(rename = "requestBodies", default)]
    pub request_bodies: HashMap<String, RequestBody>,
    #[serde(default)]
    pub headers: HashMap<String, OpenApiHeader>,
}
