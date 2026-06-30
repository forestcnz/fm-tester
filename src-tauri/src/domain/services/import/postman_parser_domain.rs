use crate::domain::models::import::PostmanCollection;

pub fn parse_postman(content: &str) -> Result<PostmanCollection, String> {
    let collection: PostmanCollection =
        serde_json::from_str(content).map_err(|e| format!("Postman JSON 解析失败: {}", e))?;

    let schema = collection.info.schema.as_str();
    if !schema.contains("collection/v2.") {
        return Err(format!(
            "不支持的 Postman Collection 格式: {}，仅支持 v2.0/v2.1",
            schema
        ));
    }

    Ok(collection)
}
