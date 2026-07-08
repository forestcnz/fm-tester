use crate::domain::models::import::ParsedCurl;
use crate::domain::models::{FileInfo, FormField, Header};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

pub fn parse_curl_command(input: &str) -> Result<ParsedCurl, String> {
    let merged = merge_multiline(input);
    let trimmed = merged.trim();

    if !trimmed.starts_with("curl") {
        return Err("不是有效的 cURL 命令，必须以 'curl' 开头".to_string());
    }

    let args = split_args(trimmed)?;
    parse_args(&args)
}

fn merge_multiline(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            // bash 格式：\ 后跟可选空格 + 换行符（行继续）
            while let Some(&next) = chars.peek() {
                if next == ' ' || next == '\t' {
                    chars.next();
                } else {
                    break;
                }
            }

            if let Some(&next) = chars.peek() {
                if next == '\n' {
                    chars.next();
                    result.push(' ');
                } else if next == '\r' {
                    chars.next();
                    if let Some(&next) = chars.peek() {
                        if next == '\n' {
                            chars.next();
                        }
                    }
                    result.push(' ');
                } else {
                    // 不是换行符，保留 \（后续作为转义处理）
                    result.push('\\');
                }
            } else {
                result.push('\\');
            }
        } else if ch == '^' {
            // CMD 格式：
            // 1. ^ 后跟空格+换行符 = 行继续
            // 2. ^" = 转义的双引号（Chrome 用 ^" 表示字符串边界）
            // 3. ^\ = 转义的反斜杠

            // 先跳过空格
            while let Some(&next) = chars.peek() {
                if next == ' ' || next == '\t' {
                    chars.next();
                } else {
                    break;
                }
            }

            if let Some(&next) = chars.peek() {
                if next == '\n' {
                    // 行继续：^ + 换行符
                    chars.next();
                    result.push(' ');
                } else if next == '\r' {
                    chars.next();
                    if let Some(&next) = chars.peek() {
                        if next == '\n' {
                            chars.next();
                        }
                    }
                    result.push(' ');
                } else if next == '"' {
                    // ^" = 转义的双引号，转换为普通双引号
                    chars.next();
                    result.push('"');
                } else if next == '\\' {
                    // ^\ = 转义的反斜杠，转换为普通反斜杠
                    chars.next();
                    result.push('\\');
                } else {
                    // ^ + 其他字符，保留原字符（转义）
                    result.push(next);
                    chars.next();
                }
            } else {
                // 字符串结束，忽略末尾的 ^
            }
        } else {
            result.push(ch);
        }
    }

    result.trim().to_string()
}

fn split_args(input: &str) -> Result<Vec<String>, String> {
    let mut args: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                escape_next = true;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if in_single_quote {
        return Err("单引号字符串未闭合".to_string());
    }
    if in_double_quote {
        return Err("双引号字符串未闭合".to_string());
    }

    if !current.is_empty() {
        args.push(current);
    }

    Ok(args)
}

fn parse_args(args: &[String]) -> Result<ParsedCurl, String> {
    let mut result = ParsedCurl {
        method: "GET".to_string(),
        url: String::new(),
        headers: Vec::new(),
        body: None,
        body_type: None,
        form_fields: None,
    };

    let mut i = 0;
    let args_len = args.len();

    while i < args_len {
        let arg = args[i].as_str();

        match arg {
            "curl" => {
                i += 1;
                continue;
            }
            "-X" | "--request" => {
                if i + 1 >= args_len {
                    return Err(format!("{} 参数缺少值", arg));
                }
                i += 1;
                result.method = args[i].to_uppercase();
            }
            "--url" => {
                if i + 1 >= args_len {
                    return Err("--url 参数缺少值".to_string());
                }
                i += 1;
                result.url = args[i].clone();
            }
            "-H" | "--header" => {
                if i + 1 >= args_len {
                    return Err(format!("{} 参数缺少值", arg));
                }
                i += 1;
                let header_str = &args[i];
                let header = parse_header(header_str)?;
                result.headers.push(header);
            }
            "-d" | "--data" | "--data-raw" => {
                if i + 1 >= args_len {
                    return Err(format!("{} 参数缺少值", arg));
                }
                i += 1;
                let data = &args[i];
                if !data.starts_with('@') {
                    result.body = Some(data.clone());
                    result.body_type = Some("raw".to_string());
                    ensure_content_type(&mut result.headers, "application/x-www-form-urlencoded");
                }
            }
            "--data-binary" => {
                if i + 1 >= args_len {
                    return Err("--data-binary 参数缺少值".to_string());
                }
                i += 1;
                let data = &args[i];
                if data.starts_with('@') {
                } else {
                    result.body = Some(data.clone());
                    result.body_type = Some("raw".to_string());
                    ensure_content_type(&mut result.headers, "application/octet-stream");
                }
            }
            "--data-urlencode" => {
                if i + 1 >= args_len {
                    return Err("--data-urlencode 参数缺少值".to_string());
                }
                i += 1;
                let data = &args[i];
                if !data.starts_with('@') {
                    if result.body.is_none() {
                        result.body = Some(data.clone());
                    } else {
                        result.body = Some(format!("{}&{}", result.body.unwrap(), data));
                    }
                    result.body_type = Some("raw".to_string());
                    ensure_content_type(&mut result.headers, "application/x-www-form-urlencoded");
                }
            }
            "-F" | "--form" => {
                if i + 1 >= args_len {
                    return Err(format!("{} 参数缺少值", arg));
                }
                i += 1;
                let form_str = &args[i];
                let field = parse_form_field(form_str)?;
                if result.form_fields.is_none() {
                    result.form_fields = Some(Vec::new());
                }
                result.form_fields.as_mut().unwrap().push(field);
                result.body_type = Some("form-data".to_string());
            }
            "-u" | "--user" => {
                if i + 1 >= args_len {
                    return Err(format!("{} 参数缺少值", arg));
                }
                i += 1;
                let user_info = &args[i];
                let auth_header = create_basic_auth_header(user_info);
                result.headers.push(auth_header);
            }
            "-b" | "--cookie" => {
                if i + 1 >= args_len {
                    return Err(format!("{} 参数缺少值", arg));
                }
                i += 1;
                let cookie_str = &args[i];
                result.headers.push(Header {
                    key: "Cookie".to_string(),
                    value: cookie_str.clone(),
                    enabled: true,
                    description: None,
                });
            }
            "-A" | "--user-agent" => {
                if i + 1 >= args_len {
                    return Err(format!("{} 参数缺少值", arg));
                }
                i += 1;
                result.headers.push(Header {
                    key: "User-Agent".to_string(),
                    value: args[i].clone(),
                    enabled: true,
                    description: None,
                });
            }
            "-e" | "--referer" => {
                if i + 1 >= args_len {
                    return Err(format!("{} 参数缺少值", arg));
                }
                i += 1;
                result.headers.push(Header {
                    key: "Referer".to_string(),
                    value: args[i].clone(),
                    enabled: true,
                    description: None,
                });
            }
            "--compressed" => {
                result.headers.push(Header {
                    key: "Accept-Encoding".to_string(),
                    value: "gzip, deflate".to_string(),
                    enabled: true,
                    description: None,
                });
            }
            "-k" | "--insecure" | "-s" | "--silent" | "-S" | "--show-error" | "-L"
            | "--location" | "-i" | "--include" | "-v" | "--verbose" | "-f" | "--fail"
            | "--progress-bar" | "-o" | "--output" | "-O" | "--remote-name" | "-w"
            | "--write-out" | "-x" | "--proxy" | "--connect-timeout" | "--max-time" | "--retry"
            | "-n" | "--netrc" | "-I" | "--head" | "--no-keepalive" | "-q" | "--disable" | "-c"
            | "--cookie-jar" => {}
            arg if arg.starts_with('-') => {}
            _ => {
                if result.url.is_empty() && !arg.is_empty() {
                    let potential_url = arg;
                    if potential_url.contains("://")
                        || potential_url.starts_with("http://")
                        || potential_url.starts_with("https://")
                        || potential_url.starts_with('/')
                    {
                        result.url = potential_url.to_string();
                    }
                }
            }
        }

        i += 1;
    }

    if result.url.is_empty() {
        return Err("缺少 URL 参数".to_string());
    }

    if result.method == "GET" && result.body.is_some() {
        result.method = "POST".to_string();
    }

    Ok(result)
}

fn parse_header(header_str: &str) -> Result<Header, String> {
    let trimmed = header_str.trim();

    if let Some(pos) = trimmed.find(':') {
        let key = trimmed[..pos].trim().to_string();
        let value = trimmed[pos + 1..].trim().to_string();
        return Ok(Header {
            key,
            value,
            enabled: true,
            description: None,
        });
    }

    // 处理 curl -H "HeaderName;" 无值 header（Chrome DevTools 会导出 Authorization;）
    if let Some(stripped) = trimmed.strip_suffix(';') {
        let key = stripped.trim().to_string();
        if !key.is_empty() {
            return Ok(Header {
                key,
                value: String::new(),
                enabled: true,
                description: None,
            });
        }
    }

    Err(format!("无效的 header 格式: {}", header_str))
}

fn parse_form_field(form_str: &str) -> Result<FormField, String> {
    let eq_pos = form_str.find('=');
    if eq_pos.is_none() {
        return Err(format!("无效的 form 字段格式: {}", form_str));
    }

    let pos = eq_pos.unwrap();
    let key = form_str[..pos].to_string();
    let value_part = &form_str[pos + 1..];

    if let Some(file_path) = value_part.strip_prefix('@') {
        let file_name = file_path
            .split('/')
            .next_back()
            .or_else(|| file_path.split('\\').next_back())
            .unwrap_or(file_path);

        Ok(FormField {
            key,
            value: String::new(),
            field_type: "file".to_string(),
            enabled: true,
            files: Some(vec![FileInfo {
                path: file_path.to_string(),
                name: file_name.to_string(),
            }]),
        })
    } else {
        Ok(FormField {
            key,
            value: value_part.to_string(),
            field_type: "text".to_string(),
            enabled: true,
            files: None,
        })
    }
}

fn create_basic_auth_header(user_info: &str) -> Header {
    let encoded = BASE64.encode(user_info.as_bytes());
    Header {
        key: "Authorization".to_string(),
        value: format!("Basic {}", encoded),
        enabled: true,
        description: None,
    }
}

fn ensure_content_type(headers: &mut Vec<Header>, content_type: &str) {
    let has_content_type = headers
        .iter()
        .any(|h| h.key.to_lowercase() == "content-type");

    if !has_content_type {
        headers.push(Header {
            key: "Content-Type".to_string(),
            value: content_type.to_string(),
            enabled: true,
            description: None,
        });
    }
}
