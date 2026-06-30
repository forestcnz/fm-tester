use crate::domain::models::common::generate_id;
use crate::domain::models::import::{
    PostmanAuth, PostmanBody, PostmanCollection, PostmanFormDataParam, PostmanHost,
    PostmanItemOrGroup, PostmanPath, PostmanRequest, PostmanUrl, PostmanUrlObject,
};
use crate::domain::models::{Collection, FormField, Header, Variable};

pub fn convert_postman_to_collection(
    postman: PostmanCollection,
    root_name: Option<String>,
) -> Result<Collection, String> {
    let name = root_name.unwrap_or_else(|| postman.info.name.clone());

    let collection_variables = convert_variables(&postman.variable);

    let mut headers = Vec::new();
    if let Some(auth) = &postman.auth {
        if let Some(auth_headers) = convert_auth_to_headers(auth) {
            headers.extend(auth_headers);
        }
    }

    let mut children = Vec::new();
    flatten_items(&postman.item, &postman.auth, &mut children);

    Ok(Collection {
        id: generate_id("col"),
        name,
        description: postman.info.description.clone(),
        item_type: "collection".to_string(),
        children,
        method: None,
        url: None,
        params: None,
        headers: if headers.is_empty() {
            None
        } else {
            Some(headers)
        },
        body: None,
        body_type: None,
        form_fields: None,
        saved_responses: None,
        common_headers: None,
        collection_variables,
        ws_config: None,
    })
}

fn flatten_items(
    items: &[PostmanItemOrGroup],
    parent_auth: &Option<PostmanAuth>,
    result: &mut Vec<Collection>,
) {
    for item in items {
        match item {
            PostmanItemOrGroup::Item(api_item) => {
                let auth = api_item.auth.as_ref().or(parent_auth.as_ref());
                let api = convert_item_to_api(api_item, auth);
                result.push(api);
            }
            PostmanItemOrGroup::Group(group) => {
                let group_auth = group.auth.as_ref().or(parent_auth.as_ref());
                flatten_items(&group.item, &group_auth.cloned(), result);
            }
        }
    }
}

fn convert_item_to_api(
    item: &crate::domain::models::import::PostmanItem,
    auth: Option<&PostmanAuth>,
) -> Collection {
    let name = item.name.clone().unwrap_or_else(|| "Unnamed".to_string());

    let (method, url, req_headers, body, body_type, form_fields) =
        extract_request_data(&item.request, auth);

    Collection {
        id: generate_id("col"),
        name,
        description: item.description.clone(),
        item_type: "api".to_string(),
        children: Vec::new(),
        method: Some(method),
        url: Some(url),
        params: None,
        headers: Some(req_headers),
        body,
        body_type,
        form_fields,
        saved_responses: None,
        common_headers: None,
        collection_variables: convert_variables(&item.variable),
        ws_config: None,
    }
}

fn extract_request_data(
    request: &PostmanRequest,
    auth: Option<&PostmanAuth>,
) -> (
    String,
    String,
    Vec<Header>,
    Option<String>,
    Option<String>,
    Option<Vec<FormField>>,
) {
    match request {
        PostmanRequest::String(url_str) => {
            let mut headers = Vec::new();
            if let Some(auth) = auth {
                if let Some(auth_headers) = convert_auth_to_headers(auth) {
                    headers.extend(auth_headers);
                }
            }
            (
                "GET".to_string(),
                url_str.clone(),
                headers,
                None,
                None,
                None,
            )
        }
        PostmanRequest::Object(req_obj) => {
            let method = req_obj.method.clone();

            let url = extract_url(&req_obj.url);

            let mut headers = convert_headers_from_postman(&req_obj.header);
            if let Some(auth) = auth {
                if let Some(auth_headers) = convert_auth_to_headers(auth) {
                    headers.extend(auth_headers);
                }
            }

            let (body, body_type, form_fields, content_type) = extract_body(&req_obj.body);

            if let Some(ct) = content_type {
                if !headers
                    .iter()
                    .any(|h| h.key.to_lowercase() == "content-type")
                {
                    headers.push(Header {
                        key: "Content-Type".to_string(),
                        value: ct,
                        enabled: true,
                        description: None,
                    });
                }
            }

            (method, url, headers, body, body_type, form_fields)
        }
    }
}

fn extract_url(url: &Option<PostmanUrl>) -> String {
    match url {
        None => "".to_string(),
        Some(PostmanUrl::String(s)) => s.clone(),
        Some(PostmanUrl::Object(url_obj)) => {
            if let Some(raw) = &url_obj.raw {
                return raw.clone();
            }

            build_url_from_object(url_obj)
        }
    }
}

fn build_url_from_object(url_obj: &PostmanUrlObject) -> String {
    let protocol = url_obj
        .protocol
        .clone()
        .unwrap_or_else(|| "http".to_string());

    let host = match &url_obj.host {
        None => "".to_string(),
        Some(PostmanHost::String(s)) => s.clone(),
        Some(PostmanHost::Array(arr)) => arr.join("."),
    };

    let port = url_obj.port.clone().unwrap_or_default();

    let path = match &url_obj.path {
        None => "".to_string(),
        Some(PostmanPath::String(s)) => {
            if s.starts_with('/') {
                s.clone()
            } else {
                format!("/{}", s)
            }
        }
        Some(PostmanPath::Array(arr)) => {
            if arr.is_empty() {
                "".to_string()
            } else {
                format!("/{}", arr.join("/"))
            }
        }
    };

    let query = if url_obj.query.is_empty() {
        "".to_string()
    } else {
        let params: Vec<String> = url_obj
            .query
            .iter()
            .filter(|q| !q.disabled)
            .filter_map(|q| {
                let key = q.key.clone()?;
                let value = q.value.clone().unwrap_or_default();
                Some(format!("{}={}", key, value))
            })
            .collect();
        if params.is_empty() {
            "".to_string()
        } else {
            format!("?{}", params.join("&"))
        }
    };

    let port_part = if port.is_empty() {
        "".to_string()
    } else {
        format!(":{}", port)
    };

    format!("{}://{}{}{}{}", protocol, host, port_part, path, query)
}

fn extract_body(
    body: &Option<PostmanBody>,
) -> (
    Option<String>,
    Option<String>,
    Option<Vec<FormField>>,
    Option<String>,
) {
    match body {
        None => (None, None, None, None),
        Some(b) => match b.mode.as_deref() {
            Some("raw") => {
                let content_type = b
                    .options
                    .as_ref()
                    .and_then(|o| o.raw.as_ref())
                    .and_then(|r| r.language.as_ref())
                    .map(|l| match l.as_str() {
                        "json" => "application/json",
                        "xml" => "application/xml",
                        "html" => "text/html",
                        "text" => "text/plain",
                        _ => "text/plain",
                    })
                    .unwrap_or("application/json");

                (
                    b.raw.clone(),
                    Some("raw".to_string()),
                    None,
                    Some(content_type.to_string()),
                )
            }
            Some("urlencoded") => {
                let fields: Vec<FormField> = b
                    .urlencoded
                    .iter()
                    .filter(|p| !p.disabled)
                    .filter_map(|p| {
                        let key = p.key.clone()?;
                        Some(FormField {
                            key,
                            value: p.value.clone().unwrap_or_default(),
                            field_type: "text".to_string(),
                            enabled: true,
                            files: None,
                        })
                    })
                    .collect();
                (
                    None,
                    Some("form-data".to_string()),
                    Some(fields),
                    Some("application/x-www-form-urlencoded".to_string()),
                )
            }
            Some("formdata") => {
                let fields: Vec<FormField> = b
                    .formdata
                    .iter()
                    .filter(|p| match p {
                        PostmanFormDataParam::Text { disabled, .. } => !disabled,
                        PostmanFormDataParam::File { disabled, .. } => !disabled,
                    })
                    .map(|p| match p {
                        PostmanFormDataParam::Text { key, value, .. } => FormField {
                            key: key.clone(),
                            value: value.clone().unwrap_or_default(),
                            field_type: "text".to_string(),
                            enabled: true,
                            files: None,
                        },
                        PostmanFormDataParam::File { key, .. } => FormField {
                            key: key.clone(),
                            value: "".to_string(),
                            field_type: "file".to_string(),
                            enabled: true,
                            files: None,
                        },
                    })
                    .collect();
                (
                    None,
                    Some("form-data".to_string()),
                    Some(fields),
                    Some("multipart/form-data".to_string()),
                )
            }
            Some("file") => (None, Some("binary".to_string()), None, None),
            Some("graphql") => {
                let graphql_body = b.raw.clone().unwrap_or_default();
                (
                    Some(graphql_body),
                    Some("raw".to_string()),
                    None,
                    Some("application/json".to_string()),
                )
            }
            _ => (None, None, None, None),
        },
    }
}

fn convert_headers_from_postman(
    headers: &[crate::domain::models::import::PostmanHeader],
) -> Vec<Header> {
    headers
        .iter()
        .filter(|h| !h.disabled)
        .map(|h| Header {
            key: h.key.clone(),
            value: h.value.clone().unwrap_or_default(),
            enabled: !h.disabled,
            description: None,
        })
        .collect()
}

fn convert_variables(
    vars: &[crate::domain::models::import::PostmanVariable],
) -> Option<Vec<Variable>> {
    if vars.is_empty() {
        return None;
    }

    let variables: Vec<Variable> = vars
        .iter()
        .filter_map(|v| {
            let key = v.key.clone().or(v.id.clone())?;
            Some(Variable {
                key,
                value: v.value.clone().unwrap_or_default(),
                enabled: true,
                description: v.description.clone(),
            })
        })
        .collect();

    if variables.is_empty() {
        None
    } else {
        Some(variables)
    }
}

fn convert_auth_to_headers(auth: &PostmanAuth) -> Option<Vec<Header>> {
    match auth.auth_type.as_str() {
        "bearer" => {
            let token = auth
                .bearer
                .iter()
                .find(|a| a.key.as_deref() == Some("token"))
                .and_then(|a| a.value.clone())
                .unwrap_or_else(|| "{{bearerToken}}".to_string());

            Some(vec![Header {
                key: "Authorization".to_string(),
                value: format!("Bearer {}", token),
                enabled: true,
                description: None,
            }])
        }
        "basic" => Some(vec![Header {
            key: "Authorization".to_string(),
            value: "Basic {{basicAuth}}".to_string(),
            enabled: true,
            description: None,
        }]),
        "apikey" => {
            let key_name = auth
                .apikey
                .iter()
                .find(|a| a.key.as_deref() == Some("key"))
                .and_then(|a| a.value.clone())
                .unwrap_or_else(|| "X-API-Key".to_string());
            let key_value = auth
                .apikey
                .iter()
                .find(|a| a.key.as_deref() == Some("value"))
                .and_then(|a| a.value.clone())
                .unwrap_or_else(|| "{{apiKey}}".to_string());

            Some(vec![Header {
                key: key_name,
                value: key_value,
                enabled: true,
                description: None,
            }])
        }
        _ => None,
    }
}
