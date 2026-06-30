use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanCollection {
    pub info: PostmanInfo,
    pub item: Vec<PostmanItemOrGroup>,
    #[serde(default)]
    pub variable: Vec<PostmanVariable>,
    #[serde(default)]
    pub auth: Option<PostmanAuth>,
    #[serde(default)]
    pub event: Vec<PostmanEvent>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanInfo {
    #[serde(rename = "_postman_id", default)]
    pub postman_id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub schema: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PostmanItemOrGroup {
    Item(PostmanItem),
    Group(PostmanItemGroup),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanItem {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub request: PostmanRequest,
    #[serde(default)]
    pub response: Vec<PostmanResponse>,
    #[serde(default)]
    pub variable: Vec<PostmanVariable>,
    #[serde(default)]
    pub event: Vec<PostmanEvent>,
    #[serde(default)]
    pub auth: Option<PostmanAuth>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanItemGroup {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub item: Vec<PostmanItemOrGroup>,
    #[serde(default)]
    pub variable: Vec<PostmanVariable>,
    #[serde(default)]
    pub auth: Option<PostmanAuth>,
    #[serde(default)]
    pub event: Vec<PostmanEvent>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PostmanRequest {
    String(String),
    Object(PostmanRequestObject),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanRequestObject {
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub url: Option<PostmanUrl>,
    #[serde(default)]
    pub header: Vec<PostmanHeader>,
    #[serde(default)]
    pub body: Option<PostmanBody>,
    #[serde(default)]
    pub auth: Option<PostmanAuth>,
    #[serde(default)]
    pub description: Option<String>,
}

fn default_method() -> String {
    "GET".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PostmanUrl {
    String(String),
    Object(PostmanUrlObject),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanUrlObject {
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub host: Option<PostmanHost>,
    #[serde(default)]
    pub path: Option<PostmanPath>,
    #[serde(default)]
    pub port: Option<String>,
    #[serde(default)]
    pub query: Vec<PostmanQueryParam>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PostmanHost {
    String(String),
    Array(Vec<String>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PostmanPath {
    String(String),
    Array(Vec<String>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanQueryParam {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanHeader {
    pub key: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanBody {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub urlencoded: Vec<PostmanUrlEncodedParam>,
    #[serde(default)]
    pub formdata: Vec<PostmanFormDataParam>,
    #[serde(default)]
    pub options: Option<PostmanBodyOptions>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanUrlEncodedParam {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PostmanFormDataParam {
    Text {
        key: String,
        #[serde(default)]
        value: Option<String>,
        #[serde(default)]
        disabled: bool,
        #[serde(rename = "type", default)]
        param_type: Option<String>,
    },
    File {
        key: String,
        #[serde(default)]
        src: Option<String>,
        #[serde(default)]
        disabled: bool,
        #[serde(rename = "type", default)]
        param_type: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanBodyOptions {
    #[serde(default)]
    pub raw: Option<PostmanRawOptions>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanRawOptions {
    #[serde(default)]
    pub language: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanVariable {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(rename = "type", default)]
    pub var_type: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanAuth {
    #[serde(rename = "type")]
    pub auth_type: String,
    #[serde(default)]
    pub bearer: Vec<PostmanAuthAttribute>,
    #[serde(default)]
    pub basic: Vec<PostmanAuthAttribute>,
    #[serde(default)]
    pub apikey: Vec<PostmanAuthAttribute>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanAuthAttribute {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanEvent {
    pub listen: String,
    #[serde(default)]
    pub script: Option<PostmanScript>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanScript {
    #[serde(default)]
    pub exec: Option<PostmanScriptExec>,
    #[serde(rename = "type", default)]
    pub script_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PostmanScriptExec {
    String(String),
    Array(Vec<String>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanResponse {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub code: Option<u16>,
    #[serde(default)]
    pub header: Vec<PostmanHeader>,
    #[serde(default)]
    pub body: Option<String>,
}
