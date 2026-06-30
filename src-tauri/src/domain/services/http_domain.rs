//! HTTP 领域服务
//!
//! 提供 HTTP 相关的纯业务逻辑（Cookie 解析、Shell 转义）。
//! 响应时间统计已移到 Application 层。

use crate::domain::models::Cookie;
use chrono::Local;

/// 解析 Set-Cookie header
pub fn parse_set_cookie(cookie_str: &str, default_domain: &str) -> Result<Cookie, String> {
    let parts: Vec<&str> = cookie_str.split(';').collect();
    let name_value = parts.first().unwrap_or(&"");

    let (name, value) = name_value.split_once('=').unwrap_or(("", ""));

    let mut domain = default_domain.to_string();
    let mut path = "/".to_string();
    let mut secure = false;
    let mut http_only = false;
    let mut expires = None;
    let mut max_age = None;

    for part in parts.iter().skip(1) {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("Domain=") {
            domain = rest.to_string();
            // 移除前导点
            if domain.starts_with('.') {
                domain = domain[1..].to_string();
            }
        } else if let Some(rest) = part.strip_prefix("Path=") {
            path = rest.to_string();
        } else if part == "Secure" {
            secure = true;
        } else if part == "HttpOnly" {
            http_only = true;
        } else if let Some(rest) = part.strip_prefix("Expires=") {
            expires = Some(rest.to_string());
        } else if let Some(rest) = part.strip_prefix("Max-Age=") {
            max_age = rest.parse::<u64>().ok();
        }
    }

    Ok(Cookie {
        name: name.to_string(),
        value: value.to_string(),
        domain,
        path,
        expires,
        max_age,
        secure,
        http_only,
        created_at: Local::now().to_rfc3339(),
    })
}

/// Shell 转义函数（适用于 Windows 和 Unix）
pub fn shell_escape(s: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        if s.contains('"') {
            format!("\"{}\"", s.replace('"', "\\\""))
        } else if s.contains(' ')
            || s.contains('&')
            || s.contains('|')
            || s.contains('<')
            || s.contains('>')
        {
            format!("\"{}\"", s)
        } else {
            s.to_string()
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if s.contains('\'') {
            format!("'{}'", s.replace("'", "'\"'\"'"))
        } else if s.contains(' ')
            || s.contains('&')
            || s.contains('|')
            || s.contains('<')
            || s.contains('>')
            || s.contains('$')
        {
            format!("'{}'", s)
        } else {
            s.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------- parse_set_cookie --------------------

    #[test]
    fn parse_set_cookie_basic() {
        let cookie = parse_set_cookie("session=abc123; Path=/; HttpOnly", "example.com").unwrap();
        assert_eq!(cookie.name, "session");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.path, "/");
        assert!(cookie.http_only);
        assert!(!cookie.secure);
    }

    #[test]
    fn parse_set_cookie_domain_strips_leading_dot_and_flags() {
        // Domain 前导点应被移除；Secure/HttpOnly/Max-Age 应正确解析
        let cookie = parse_set_cookie(
            "token=xyz; Domain=.api.example.com; Secure; HttpOnly; Max-Age=3600",
            "fallback.com",
        )
        .unwrap();
        assert_eq!(cookie.domain, "api.example.com");
        assert!(cookie.secure);
        assert!(cookie.http_only);
        assert_eq!(cookie.max_age, Some(3600));
    }

    #[test]
    fn parse_set_cookie_falls_back_to_defaults() {
        // 未指定 Domain/Path 时使用默认值
        let cookie = parse_set_cookie("k=v", "default.com").unwrap();
        assert_eq!(cookie.domain, "default.com");
        assert_eq!(cookie.path, "/");
        assert_eq!(cookie.expires, None);
        assert_eq!(cookie.max_age, None);
    }

    #[test]
    fn parse_set_cookie_empty_value() {
        let cookie = parse_set_cookie("empty=; Path=/", "x.com").unwrap();
        assert_eq!(cookie.value, "");
    }

    // -------------------- shell_escape --------------------

    #[test]
    fn shell_escape_plain_string_unchanged() {
        // 无特殊字符的纯字符串应原样返回（两平台行为一致）
        assert_eq!(shell_escape("hello"), "hello");
    }

    #[test]
    fn shell_escape_keeps_content_visible_with_spaces() {
        // 含空格的字符串必然被引号包裹，但原始内容必须仍然可读
        let escaped = shell_escape("hello world");
        assert!(escaped.contains("hello world"));
    }
}
