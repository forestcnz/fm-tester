use crate::domain::models::import::OpenApi3;

pub fn parse_openapi(content: &str, format: &str) -> Result<OpenApi3, String> {
    let openapi = if format == "yaml" || (format == "auto" && content.trim().starts_with("---")) {
        serde_yaml_ng::from_str(content).map_err(|e| format!("YAML 解析失败: {}", e))?
    } else {
        serde_json::from_str(content).map_err(|e| format!("JSON 解析失败: {}", e))?
    };
    Ok(openapi)
}
