//! HTTP 客户端基础设施
//!
//! 自实现连接各阶段（DNS / TCP / TLS / HTTP）以提供 DevTools 风格的耗时分解。
//! HTTP 协议层复用 hyper（与 reqwest 0.11 底层一致），连接建立由本模块控制以便插入计时点。

use crate::domain::models::{FormField, Header, RequestTiming};
use hyper::client::conn::Builder;
use hyper::{Body, Method, Request, Response};
use rustls::{ClientConfig, RootCertStore, ServerName};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{lookup_host, TcpStream};
use tokio_rustls::TlsConnector;
use url::Url;

/// 分阶段计时的响应结果
pub struct TimingResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub final_url: String,
    pub body: Body,
    pub timing: RequestTiming,
}

/// 请求体构造结果：(body, content_type)
type BuiltBody = (Option<Body>, Option<String>);

/// HTTP 客户端服务
pub struct HttpClientService;

impl HttpClientService {
    pub fn new() -> Self {
        Self
    }

    /// 发送请求并返回分阶段计时的响应（不读取 body）
    pub async fn send_with_timing(
        method: &str,
        url: &str,
        headers: Vec<Header>,
        body: Option<String>,
        body_type: Option<String>,
        form_fields: Option<Vec<FormField>>,
        timeout_ms: u64,
    ) -> Result<TimingResponse, String> {
        let timeout = Duration::from_millis(timeout_ms);
        let method_upper = method.to_uppercase();

        // 循环 follow 重定向，只保留最终一跳的分阶段计时
        let mut current_url = url.to_string();
        let mut redirects = 0u32;

        loop {
            if redirects > 10 {
                return Err("重定向次数过多".to_string());
            }

            let result = tokio::time::timeout(
                timeout,
                send_once(
                    &method_upper,
                    &current_url,
                    &headers,
                    body_type.as_deref(),
                    body.as_deref(),
                    form_fields.as_deref(),
                ),
            )
            .await;

            let (resp, timing) = match result {
                Ok(Ok(v)) => v,
                Ok(Err(e)) => return Err(e),
                Err(_) => return Err("请求超时".to_string()),
            };

            // 处理重定向（3xx 且有 Location）
            let location = if (300..400).contains(&resp.status().as_u16()) {
                resp.headers()
                    .get("location")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
            } else {
                None
            };

            if let Some(loc) = location {
                // drain 重定向响应 body，避免连接泄漏
                let _ = hyper::body::aggregate(resp).await;
                let next = resolve_redirect(&current_url, &loc)?;
                current_url = next;
                redirects += 1;
                continue;
            }

            return build_timing_response(resp, current_url, timing);
        }
    }
}

impl Default for HttpClientService {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行单次请求（一次连接），返回响应与分阶段计时
async fn send_once(
    method: &str,
    url: &str,
    headers: &[Header],
    body_type: Option<&str>,
    body_str: Option<&str>,
    form_fields: Option<&[FormField]>,
) -> Result<(Response<Body>, RequestTiming), String> {
    let has_body = !matches!(method, "GET" | "HEAD");
    let (req_body, body_content_type) = if has_body {
        build_request_body(body_type, body_str, form_fields.map(|f| f.to_vec()))?
    } else {
        (None, None)
    };

    let parsed = Url::parse(url).map_err(|e| format!("URL 解析失败: {}", e))?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(format!("不支持的协议: {}", scheme));
    }
    let is_https = scheme == "https";
    let host = parsed
        .host_str()
        .ok_or_else(|| "URL 缺少主机名".to_string())?;
    let port = parsed
        .port_or_known_default()
        .ok_or_else(|| "无法确定端口".to_string())?;
    let authority = format!("{}:{}", host, port);

    let path = parsed.path();
    let path = if path.is_empty() { "/" } else { path };
    let query = parsed
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();
    let uri_path = format!("{}{}", path, query);

    let mut timing = RequestTiming::default();

    // ===== DNS =====
    let t = Instant::now();
    let mut addrs = lookup_host((host, port))
        .await
        .map_err(|e| format!("DNS 解析失败: {}", e))?;
    timing.dns_ms = t.elapsed().as_millis() as u64;

    // ===== TCP =====
    let t = Instant::now();
    let tcp_stream = match addrs.next() {
        Some(addr) => TcpStream::connect(&addr)
            .await
            .map_err(|e| format!("TCP 连接失败: {}", e))?,
        None => return Err("DNS 解析未返回地址".to_string()),
    };
    timing.connect_ms = t.elapsed().as_millis() as u64;

    // ===== 发送请求（https 走 TLS，http 直连）=====
    let resp = if is_https {
        let t = Instant::now();
        let (tls_stream, alpn_h2) = connect_tls(tcp_stream, host).await?;
        timing.tls_ms = t.elapsed().as_millis() as u64;

        let t = Instant::now();
        let resp = send_over_connection(
            tls_stream,
            alpn_h2,
            method,
            &uri_path,
            &authority,
            headers,
            req_body,
            body_content_type.as_deref(),
        )
        .await?;
        timing.ttfb_ms = t.elapsed().as_millis() as u64;
        resp
    } else {
        let t = Instant::now();
        let resp = send_over_connection(
            tcp_stream,
            false,
            method,
            &uri_path,
            &authority,
            headers,
            req_body,
            body_content_type.as_deref(),
        )
        .await?;
        timing.ttfb_ms = t.elapsed().as_millis() as u64;
        resp
    };

    Ok((resp, timing))
}

/// 建立 TLS 连接，返回流与 ALPN 协商结果（true=h2）
async fn connect_tls(
    tcp_stream: TcpStream,
    host: &str,
) -> Result<(tokio_rustls::client::TlsStream<TcpStream>, bool), String> {
    let mut root_store = RootCertStore::empty();
    let certs = rustls_native_certs::load_native_certs()
        .map_err(|e| format!("加载系统根证书失败: {}", e))?;
    for cert in certs {
        root_store
            .add(&rustls::Certificate(cert.0))
            .map_err(|e| format!("添加根证书失败: {}", e))?;
    }

    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let config = {
        let mut cfg = config;
        cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        cfg
    };
    let connector = TlsConnector::from(Arc::new(config));

    let server_name =
        ServerName::try_from(host).map_err(|_| format!("无效的 TLS 主机名: {}", host))?;

    let tls_stream = connector
        .connect(server_name, tcp_stream)
        .await
        .map_err(|e| format!("TLS 握手失败: {}", e))?;

    let (_, session) = tls_stream.get_ref();
    let alpn_h2 = session.alpn_protocol().map(|p| p == b"h2").unwrap_or(false);

    Ok((tls_stream, alpn_h2))
}

/// 在已建立的连接上通过 hyper 发送 HTTP 请求
async fn send_over_connection<T>(
    stream: T,
    is_h2: bool,
    method: &str,
    uri_path: &str,
    authority: &str,
    headers: &[Header],
    body: Option<Body>,
    body_content_type: Option<&str>,
) -> Result<Response<Body>, String>
where
    T: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    let mut builder = Builder::new();
    if is_h2 {
        builder.http2_only(true);
    }
    let (mut sender, connection) = builder
        .handshake(stream)
        .await
        .map_err(|e| format!("建立 HTTP 连接失败: {}", e))?;
    // 后台驱动连接
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let hyper_method = match method {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "PATCH" => Method::PATCH,
        "HEAD" => Method::HEAD,
        "OPTIONS" => Method::OPTIONS,
        _ => Method::GET,
    };

    let mut req_builder = Request::builder()
        .method(hyper_method)
        .uri(uri_path)
        .header(hyper::header::HOST, authority);

    // body content-type
    if let Some(ct) = body_content_type {
        req_builder = req_builder.header(hyper::header::CONTENT_TYPE, ct);
    }

    // 用户自定义 headers
    for header in headers {
        if header.enabled && !header.key.trim().is_empty() {
            req_builder = req_builder.header(&header.key, &header.value);
        }
    }

    let req = req_builder
        .body(body.unwrap_or_default())
        .map_err(|e| format!("构造请求失败: {}", e))?;

    sender
        .send_request(req)
        .await
        .map_err(|e| format!("发送请求失败: {}", e))
}

/// 构造请求体，返回 (body, content_type)
fn build_request_body(
    body_type: Option<&str>,
    body: Option<&str>,
    form_fields: Option<Vec<FormField>>,
) -> Result<BuiltBody, String> {
    let actual = body_type.unwrap_or("raw");

    match actual {
        "form-data" => {
            if let Some(fields) = form_fields {
                let boundary = format!("----FmTester{:016x}", rand::random::<u64>());
                let mut buf: Vec<u8> = Vec::new();
                for field in &fields {
                    if !field.enabled || field.key.is_empty() {
                        continue;
                    }
                    match field.field_type.as_str() {
                        "text" => {
                            buf.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
                            buf.extend_from_slice(
                                format!(
                                    "Content-Disposition: form-data; name=\"{}\"\r\n\r\n",
                                    field.key
                                )
                                .as_bytes(),
                            );
                            buf.extend_from_slice(field.value.as_bytes());
                            buf.extend_from_slice(b"\r\n");
                        }
                        "file" => {
                            if let Some(ref files) = field.files {
                                for file_info in files {
                                    let path = std::path::Path::new(&file_info.path);
                                    if !path.exists() {
                                        continue;
                                    }
                                    let bytes = std::fs::read(&file_info.path)
                                        .map_err(|e| format!("读取文件失败: {}", e))?;
                                    buf.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
                                    buf.extend_from_slice(
                                        format!(
                                            "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
                                            field.key, file_info.name
                                        )
                                        .as_bytes(),
                                    );
                                    buf.extend_from_slice(
                                        "Content-Type: application/octet-stream\r\n\r\n".as_bytes(),
                                    );
                                    buf.extend_from_slice(&bytes);
                                    buf.extend_from_slice(b"\r\n");
                                }
                            }
                        }
                        _ => {}
                    }
                }
                buf.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());
                let ct = format!("multipart/form-data; boundary={}", boundary);
                Ok((Some(Body::from(buf)), Some(ct)))
            } else {
                Ok((None, None))
            }
        }
        "x-www-form-urlencoded" => {
            if let Some(fields) = form_fields {
                let mut pairs = Vec::new();
                for field in &fields {
                    if field.enabled && !field.key.is_empty() {
                        pairs.push(format!(
                            "{}={}",
                            urlencoding::encode(&field.key),
                            urlencoding::encode(&field.value)
                        ));
                    }
                }
                let body_str = pairs.join("&");
                Ok((
                    Some(Body::from(body_str)),
                    Some("application/x-www-form-urlencoded".to_string()),
                ))
            } else {
                Ok((None, None))
            }
        }
        "binary" => {
            if let Some(fields) = form_fields {
                for field in &fields {
                    if field.enabled && field.field_type == "file" {
                        if let Some(ref files) = field.files {
                            if let Some(file_info) = files.first() {
                                let path = std::path::Path::new(&file_info.path);
                                if path.exists() {
                                    let bytes = std::fs::read(&file_info.path)
                                        .map_err(|e| format!("读取文件失败: {}", e))?;
                                    return Ok((
                                        Some(Body::from(bytes)),
                                        Some("application/octet-stream".to_string()),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            Ok((None, None))
        }
        _ => {
            // raw
            if let Some(b) = body {
                if !b.is_empty() {
                    Ok((Some(Body::from(b.to_string())), None))
                } else {
                    Ok((None, None))
                }
            } else {
                Ok((None, None))
            }
        }
    }
}

/// 解析重定向 Location（相对或绝对）
fn resolve_redirect(base: &str, location: &str) -> Result<String, String> {
    if location.starts_with("http://") || location.starts_with("https://") {
        return Ok(location.to_string());
    }
    let base_url = Url::parse(base).map_err(|e| format!("解析基础 URL 失败: {}", e))?;
    base_url
        .join(location)
        .map(|u| u.to_string())
        .map_err(|e| format!("解析重定向地址失败: {}", e))
}

/// 将 hyper Response 转为 TimingResponse
fn build_timing_response(
    resp: Response<Body>,
    final_url: String,
    mut timing: RequestTiming,
) -> Result<TimingResponse, String> {
    let status = resp.status().as_u16();
    let status_text = resp.status().canonical_reason().unwrap_or("").to_string();

    let (parts, body) = resp.into_parts();
    let headers: HashMap<String, String> = parts
        .headers
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    timing.total_ms = timing.dns_ms + timing.connect_ms + timing.tls_ms + timing.ttfb_ms;
    // download_ms 由调用方读完 body 后填充，total_ms 随之更新

    Ok(TimingResponse {
        status,
        status_text,
        headers,
        final_url,
        body,
        timing,
    })
}
