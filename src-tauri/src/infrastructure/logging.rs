//! 日志初始化模块
//!
//! - 日志级别通过 `./config/config.env` 的 `LOG_LEVEL` 配置，默认 `info`
//! - 当天日志写入 `./data/logs/fm-tester.log`
//! - 启动时将前一天的日志归档到 `./data/logs/YYYY-MM-DD/fm-tester.log`
//! - 自动清理超过 7 天的日期目录

use std::fs;
use std::path::Path;

use chrono::Datelike;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::prelude::*;

use crate::infrastructure::data_dir;

/// 本地时间计时器，用于日志时间戳
struct LocalTimer;

impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(
            w,
            "{}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f")
        )
    }
}

const DEFAULT_LOG_LEVEL: &str = "info";
const KEEP_DAYS: i64 = 7;
const DATE_FMT: &str = "%Y-%m-%d";
const LOG_FILE_NAME: &str = "fm-tester.log";

const CONFIG_ENV_TEMPLATE: &str = "# FM Tester 配置文件\n\
# 日志级别可选：trace, debug, info, warn, error\n\
LOG_LEVEL=info\n";

/// 初始化全局日志订阅（按日期目录归档 + 自动清理旧日志）。
///
/// 应在应用入口最早处调用，且仅调用一次。
pub fn init() {
    let logs_dir = data_dir::get_logs_dir();
    if let Err(e) = fs::create_dir_all(&logs_dir) {
        eprintln!("创建日志目录失败: {}", e);
    }

    // 将前一天的日志归档到日期子目录
    archive_old_log(&logs_dir);

    clean_old_logs(&logs_dir);

    let level = read_log_level();
    let parsed_level = parse_level(&level);

    // 当天日志直接写入 logs/fm-tester.log
    match fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(logs_dir.join(LOG_FILE_NAME))
    {
        Ok(file) => {
            let (writer, guard) = non_blocking(file);
            // WorkerGuard 必须存活才能保证日志刷盘，forget 使其在进程生命周期内常驻
            std::mem::forget(guard);
            tracing_subscriber::registry()
                .with(parsed_level)
                .with(
                    fmt::layer()
                        .with_writer(writer)
                        .with_ansi(false)
                        .with_timer(LocalTimer),
                )
                .init();
        }
        Err(e) => {
            eprintln!("打开日志文件失败: {}, 日志将输出到 stderr", e);
            tracing_subscriber::registry()
                .with(parsed_level)
                .with(fmt::layer().with_ansi(false).with_timer(LocalTimer))
                .init();
        }
    }

    tracing::info!("日志已初始化（级别: {}）", level);
}

/// 读取 `./config/config.env` 中的 `LOG_LEVEL`。
/// 文件不存在时自动创建并写入默认模板。
fn read_log_level() -> String {
    let config_path = data_dir::get_config_env_path();

    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&config_path, CONFIG_ENV_TEMPLATE);
        return DEFAULT_LOG_LEVEL.to_string();
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return DEFAULT_LOG_LEVEL.to_string(),
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            if key.trim().eq_ignore_ascii_case("LOG_LEVEL") {
                let v = value.trim();
                if !v.is_empty() {
                    return v.to_lowercase();
                }
            }
        }
    }

    DEFAULT_LOG_LEVEL.to_string()
}

fn parse_level(s: &str) -> LevelFilter {
    match s {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "info" => LevelFilter::INFO,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    }
}

/// 将前一天的日志文件归档到日期子目录。
///
/// 检查 `logs/fm-tester.log` 的修改时间，若非今天则移动到 `logs/YYYY-MM-DD/fm-tester.log`。
fn archive_old_log(logs_dir: &Path) {
    let log_path = logs_dir.join(LOG_FILE_NAME);
    if !log_path.exists() {
        return;
    }

    let modified = match fs::metadata(&log_path).ok().and_then(|m| m.modified().ok()) {
        Some(t) => {
            let dt: chrono::DateTime<chrono::Local> = t.into();
            dt
        }
        None => return,
    };

    // 今天写的日志不归档
    if modified.date_naive() >= chrono::Local::now().date_naive() {
        return;
    }

    // 按修改日期归档到子目录
    let date_str = modified.format(DATE_FMT).to_string();
    let archive_dir = logs_dir.join(&date_str);
    let _ = fs::create_dir_all(&archive_dir);
    if fs::rename(&log_path, archive_dir.join(LOG_FILE_NAME)).is_ok() {
        eprintln!("日志已归档到 {}/", date_str);
    }
}

/// 删除超过 KEEP_DAYS 天的日期目录。
///
/// 扫描 logs 目录下的子目录名（格式 YYYY-MM-DD），
/// 将日期早于截止日的目录整体删除。
fn clean_old_logs(logs_dir: &Path) {
    let entries = match fs::read_dir(logs_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let cutoff = chrono::Local::now() - chrono::Duration::days(KEEP_DAYS);

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        let date = match chrono::NaiveDate::parse_from_str(dir_name, DATE_FMT) {
            Ok(d) => d,
            Err(_) => continue,
        };

        if date < cutoff.date_naive() {
            let _ = fs::remove_dir_all(&path);
            eprintln!(
                "清理过期日志目录: {} ({}-{:02}-{:02})",
                dir_name,
                date.year(),
                date.month(),
                date.day()
            );
        }
    }
}
