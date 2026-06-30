//! 加密服务领域接口
//!
//! 定义加密服务的领域接口，具体实现在Infrastructure层。

/// 加密服务 Trait（领域层接口定义）
pub trait EncryptionService: Send {
    /// 加密字符串
    fn encrypt(&self, plain_text: &str) -> Result<String, String>;

    /// 解密字符串
    fn decrypt(&self, encrypted: &str) -> Result<String, String>;
}
