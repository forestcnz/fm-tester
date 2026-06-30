//! 安全文件操作校验
//!
//! 提供输入校验工具，防止路径遍历攻击和无效输入。
//! 用于在将用户/前端传入的 ID、文件名、日期等值拼接进文件系统路径或 SQL 前，
//! 先做白名单校验，避免 `../`、绝对路径、分隔符等危险输入越权访问任意文件。
//!
//! 设计原则：
//! - 采用「白名单」而非黑名单，只放行明确安全的字符集；
//! - 校验失败返回可读的中文错误信息（`Result<(), String>`），与项目错误处理风格一致；
//! - 纯函数、无副作用、无 I/O，便于单元测试覆盖。

// 校验函数已由 domain::services::validation_domain 的单元测试覆盖；
// 接入 data_dir 等路径拼接处属工程化路线图后续项。为避免在非测试编译中被
// 误判为死代码而遭到剔除，此处显式放行 dead_code。
#![allow(dead_code)]

use chrono::{Datelike, NaiveDate};

/// 校验通用 ID（如 workspace_id、api_id、collection_id 等）。
///
/// 规则：
/// - 非空；
/// - 长度 ≤ 255；
/// - 仅允许 ASCII 字母、数字、下划线 `_`、连字符 `-`。
///
/// 该白名单天然拒绝 `/`、`\`、`.`、`..`、空格及所有特殊字符，
/// 从而杜绝路径遍历与非法文件名片段。
pub fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("ID 不能为空".to_string());
    }
    if id.len() > 255 {
        return Err("ID 长度超过限制（最大 255）".to_string());
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err("ID 包含非法字符（只允许字母、数字、下划线、连字符）".to_string());
    }
    Ok(())
}

/// 校验存储型文件名（如 `{api_id}.toml`、脚本文件等）。
///
/// 规则：
/// - 非空，长度 ≤ 255；
/// - 不得包含路径分隔符（`/`、`\`）、空字节 `\0` 或路径遍历片段 `..`；
/// - 扩展名必须在白名单内：`toml`、`js`、`json`、`md`；
/// - 文件名字符仅允许 ASCII 字母、数字、下划线、连字符、点。
pub fn validate_filename(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("文件名不能为空".to_string());
    }
    if name.len() > 255 {
        return Err("文件名长度超过限制（最大 255）".to_string());
    }
    // 路径遍历与分隔符拦截
    if name.contains('/') || name.contains('\\') || name.contains('\0') || name.contains("..") {
        return Err("文件名包含非法路径字符".to_string());
    }
    // 扩展名白名单：rsplit('.') 取最后一段作为扩展名
    let allowed_ext = ["toml", "js", "json", "md"];
    let ext = name.rsplit('.').next().unwrap_or("");
    if !allowed_ext.contains(&ext) {
        return Err(format!("不支持的文件扩展名（仅允许: {:?}）", allowed_ext));
    }
    // 字符白名单（含点，用于分隔扩展名）
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err("文件名包含非法字符".to_string());
    }
    Ok(())
}

/// 校验日期字符串格式为 `YYYY-MM-DD`，并验证为真实存在日期、年份在合理范围。
///
/// 规则：
/// - 严格 10 位 `YYYY-MM-DD`（拒绝 `2024-1-1`、`2024/01/01`、带时分秒等）；
/// - 必须是公历真实日期（拒绝 `2024-13-01`、`2024-01-32`）；
/// - 年份范围 1900–2100。
pub fn validate_date_format(date: &str) -> Result<(), String> {
    // 先做严格的字符级格式校验，避免 chrono 宽松解析放行「2024-01-01 00:00:00」这类带尾缀的串
    if date.len() != 10 {
        return Err("日期格式无效（要求 YYYY-MM-DD）".to_string());
    }
    let bytes = date.as_bytes();
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return Err("日期格式无效（要求 YYYY-MM-DD）".to_string());
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(i, &b)| i == 4 || i == 7 || b.is_ascii_digit())
    {
        return Err("日期格式无效（要求 YYYY-MM-DD）".to_string());
    }
    // 借助 chrono 校验日期真实性（如闰年、各月天数）
    let parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| "日期无效".to_string())?;
    let year = parsed.year();
    if !(1900..=2100).contains(&year) {
        return Err("年份超出有效范围（1900-2100）".to_string());
    }
    Ok(())
}
