//! 安全加密服务实现
//!
//! 使用 AES-256-GCM 加密，密钥随机生成并存储在安全文件中。
//! 密钥文件位置：{APP_DATA}/.encryption_key

use crate::domain::services::EncryptionService;
use crate::infrastructure::data_dir;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

/// 获取密钥文件路径
fn get_key_file_path() -> std::path::PathBuf {
    data_dir::get_key_path()
}

/// 确保密钥目录存在
fn ensure_key_dir() -> Result<(), String> {
    if let Some(parent) = get_key_file_path().parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("创建数据目录失败: {}", e))?;
        }
    }
    Ok(())
}

/// 设置文件权限（仅当前用户可读写）
fn set_file_permissions(path: &PathBuf) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .map_err(|e| format!("获取文件权限失败: {}", e))?
            .permissions();
        perms.set_mode(0o600); // 仅用户可读写
        fs::set_permissions(path, perms).map_err(|e| format!("设置文件权限失败: {}", e))?;
    }
    #[cfg(windows)]
    {
        // Windows 文件 ACL 收紧需要调用 icacls，但同步 status() 在某些环境下会挂起，
        // 且首次启动时调用 icacls 会阻塞加密服务初始化。
        // 暂时跳过 ACL 收紧，密钥已位于用户配置目录（%APPDATA%\fm-tester），
        // 默认仅当前用户可访问，安全风险可接受。
        // 后续可通过 windows-acl crate + 异步线程实现更严格的 ACL 控制。
        let _ = path;
    }
    Ok(())
}

/// 生成随机密钥
fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// 加载或创建密钥
fn load_or_create_key() -> Result<[u8; 32], String> {
    ensure_key_dir()?;
    let key_path = get_key_file_path();

    if key_path.exists() {
        // 加载现有密钥
        let mut file = fs::File::open(&key_path).map_err(|e| format!("打开密钥文件失败: {}", e))?;
        let mut key_bytes = [0u8; 32];
        file.read_exact(&mut key_bytes)
            .map_err(|e| format!("读取密钥失败: {}", e))?;
        Ok(key_bytes)
    } else {
        // 创建新密钥
        let key = generate_random_key();
        let mut file =
            fs::File::create(&key_path).map_err(|e| format!("创建密钥文件失败: {}", e))?;
        file.write_all(&key)
            .map_err(|e| format!("写入密钥失败: {}", e))?;

        // 设置文件权限
        set_file_permissions(&key_path)?;

        Ok(key)
    }
}

/// AES-GCM 加密服务实现
pub struct AesGcmEncryptionService {
    key: [u8; 32],
}

impl AesGcmEncryptionService {
    pub fn new() -> Result<Self, String> {
        let key = load_or_create_key()?;
        Ok(Self { key })
    }

    /// 生成随机 nonce（12字节）
    fn generate_nonce() -> [u8; 12] {
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        nonce
    }
}

impl Default for AesGcmEncryptionService {
    fn default() -> Self {
        Self::new().expect("初始化加密服务失败")
    }
}

impl EncryptionService for AesGcmEncryptionService {
    fn encrypt(&self, plain_text: &str) -> Result<String, String> {
        let cipher =
            Aes256Gcm::new_from_slice(&self.key).map_err(|e| format!("创建加密器失败: {}", e))?;

        // 生成随机 nonce
        let nonce_bytes = Self::generate_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密
        let ciphertext = cipher
            .encrypt(nonce, plain_text.as_bytes())
            .map_err(|e| format!("加密失败: {}", e))?;

        // 组合 nonce 和密文（格式：nonce_base64:ciphertext_base64）
        let nonce_b64 = BASE64.encode(nonce_bytes);
        let ciphertext_b64 = BASE64.encode(&ciphertext);

        Ok(format!("{}:{}", nonce_b64, ciphertext_b64))
    }

    fn decrypt(&self, encrypted: &str) -> Result<String, String> {
        // 解析格式：nonce_base64:ciphertext_base64
        let parts: Vec<&str> = encrypted.split(':').collect();
        if parts.len() != 2 {
            return Err("无效的加密数据格式".to_string());
        }

        let nonce_bytes = BASE64
            .decode(parts[0])
            .map_err(|e| format!("Nonce Base64 解码失败: {}", e))?;

        let ciphertext = BASE64
            .decode(parts[1])
            .map_err(|e| format!("密文 Base64 解码失败: {}", e))?;

        if nonce_bytes.len() != 12 {
            return Err("Nonce 长度错误".to_string());
        }

        let cipher =
            Aes256Gcm::new_from_slice(&self.key).map_err(|e| format!("创建解密器失败: {}", e))?;

        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("解密失败: {}", e))?;

        String::from_utf8(plaintext).map_err(|e| format!("UTF-8 转换失败: {}", e))
    }
}

/// 全局加密服务实例
pub fn get_encryption_service() -> AesGcmEncryptionService {
    AesGcmEncryptionService::new().expect("初始化加密服务失败")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        // 使用临时密钥文件路径
        let temp_dir = tempfile::tempdir().unwrap();
        let key_path = temp_dir.path().join(".encryption_key");

        // 手动创建密钥
        let key = generate_random_key();
        fs::write(&key_path, key).unwrap();

        let service = AesGcmEncryptionService::new().unwrap();

        let plain = "test_secret_password";
        let encrypted = service.encrypt(plain).unwrap();

        // 验证加密格式
        assert!(encrypted.contains(':'));

        let decrypted = service.decrypt(&encrypted).unwrap();
        assert_eq!(plain, decrypted);

        // 验证每次加密产生不同的密文（随机 nonce）
        let encrypted2 = service.encrypt(plain).unwrap();
        assert_ne!(encrypted, encrypted2);

        let decrypted2 = service.decrypt(&encrypted2).unwrap();
        assert_eq!(plain, decrypted2);
    }

    #[test]
    fn test_invalid_format() {
        let service = AesGcmEncryptionService::new().unwrap();

        // 无效格式
        let result = service.decrypt("invalid_format");
        assert!(result.is_err());

        // 缺少 nonce
        let result = service.decrypt("only_ciphertext");
        assert!(result.is_err());
    }
}
