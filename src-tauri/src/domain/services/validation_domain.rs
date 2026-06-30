//! 验证领域服务
//!
//! 提供输入验证功能，防止路径遍历攻击和无效输入。
//!
//! 注意：验证函数已迁移到 infrastructure::safe_file_ops 模块，
//! 此文件仅保留测试以验证 infrastructure 函数的正确性。

#[cfg(test)]
mod tests {
    use crate::infrastructure::safe_file_ops::{
        validate_date_format, validate_filename, validate_id,
    };

    #[test]
    fn test_validate_id_valid() {
        assert!(validate_id("abc123").is_ok());
        assert!(validate_id("test_id").is_ok());
        assert!(validate_id("test-id").is_ok());
        assert!(validate_id("ABC_123-xyz").is_ok());
    }

    #[test]
    fn test_validate_id_empty() {
        assert!(validate_id("").is_err());
        assert!(validate_id("").unwrap_err().contains("ID 不能为空"));
    }

    #[test]
    fn test_validate_id_too_long() {
        let long_id = "a".repeat(256);
        assert!(validate_id(&long_id).is_err());
        assert!(validate_id(&long_id).unwrap_err().contains("长度超过限制"));
    }

    #[test]
    fn test_validate_id_path_traversal() {
        assert!(validate_id("../etc/passwd").is_err());
        assert!(validate_id("test/../file").is_err());
        assert!(validate_id("test\\file").is_err());
        assert!(validate_id("test/file").is_err());
        assert!(validate_id("test\0file").is_err());
    }

    #[test]
    fn test_validate_id_invalid_chars() {
        assert!(validate_id("test@file").is_err());
        assert!(validate_id("test file").is_err());
        assert!(validate_id("test.file").is_err());
        assert!(validate_id("test#file").is_err());
    }

    #[test]
    fn test_validate_date_format_valid() {
        assert!(validate_date_format("2024-01-01").is_ok());
        assert!(validate_date_format("2024-12-31").is_ok());
        assert!(validate_date_format("2000-02-29").is_ok()); // 闰年
    }

    #[test]
    fn test_validate_date_format_invalid_format() {
        assert!(validate_date_format("2024/01/01").is_err());
        assert!(validate_date_format("01-01-2024").is_err());
        assert!(validate_date_format("2024-1-1").is_err());
        assert!(validate_date_format("2024-01-01 00:00:00").is_err());
    }

    #[test]
    fn test_validate_date_format_invalid_range() {
        assert!(validate_date_format("2024-13-01").is_err()); // 无效月份
        assert!(validate_date_format("2024-01-32").is_err()); // 无效日期
        assert!(validate_date_format("2024-00-01").is_err()); // 无效月份
        assert!(validate_date_format("1899-01-01").is_err()); // 年份太小
        assert!(validate_date_format("2101-01-01").is_err()); // 年份太大
    }

    #[test]
    fn test_validate_filename_valid() {
        assert!(validate_filename("test.toml").is_ok());
        assert!(validate_filename("test.js").is_ok());
        assert!(validate_filename("test.json").is_ok());
        assert!(validate_filename("test.md").is_ok());
        assert!(validate_filename("test_file-123.toml").is_ok());
    }

    #[test]
    fn test_validate_filename_empty() {
        assert!(validate_filename("").is_err());
        assert!(validate_filename("")
            .unwrap_err()
            .contains("文件名不能为空"));
    }

    #[test]
    fn test_validate_filename_too_long() {
        let long_name = format!("{}.toml", "a".repeat(251));
        assert!(validate_filename(&long_name).is_err());
        assert!(validate_filename(&long_name)
            .unwrap_err()
            .contains("长度超过限制"));
    }

    #[test]
    fn test_validate_filename_path_traversal() {
        assert!(validate_filename("../test.toml").is_err());
        assert!(validate_filename("test/../file.toml").is_err());
        assert!(validate_filename("test\\file.toml").is_err());
        assert!(validate_filename("test/file.toml").is_err());
        assert!(validate_filename("test\0file.toml").is_err());
    }

    #[test]
    fn test_validate_filename_invalid_extension() {
        assert!(validate_filename("test.txt").is_err());
        assert!(validate_filename("test.exe").is_err());
        assert!(validate_filename("test").is_err());
        assert!(validate_filename("test.").is_err());
    }

    #[test]
    fn test_validate_filename_invalid_chars() {
        assert!(validate_filename("test@file.toml").is_err());
        assert!(validate_filename("test file.toml").is_err());
        assert!(validate_filename("test#file.toml").is_err());
    }
}
