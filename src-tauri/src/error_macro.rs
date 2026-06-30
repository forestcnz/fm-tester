//! 统一错误消息宏

/// 统一错误消息宏
#[macro_export]
macro_rules! repo_error {
    ($msg:expr) => {
        $msg.to_string()
    };
    ($fmt:expr, $($arg:tt)*) => {
        format!($fmt, $($arg)*)
    };
}
