//! Passkey、TOTP 与会话的认证工具。

use aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::Engine;
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng, RngCore};
use sha2::{Digest, Sha256};
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

use crate::error::AppError;

const SECRET_NONCE_LEN: usize = 12;
const SECRET_PREFIX: &str = "SECv1:";

/// 生成的恢复码与其哈希。
#[derive(Debug, Clone)]
pub struct RecoveryCode {
    /// 仅展示一次的明文恢复码。
    pub plain: String,
    /// 存入数据库的 Argon2 哈希。
    pub hash: String,
}

/// 生成随机会话令牌（base64url）。
pub fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// 使用 SHA-256 哈希会话令牌用于存储。
pub fn hash_session_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// 生成一组恢复码。
pub fn generate_recovery_codes(count: usize) -> Result<Vec<RecoveryCode>, AppError> {
    let mut codes = Vec::with_capacity(count);
    for _ in 0..count {
        let mut raw = [0u8; 12];
        OsRng.fill_bytes(&mut raw);
        let plain = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw);
        let hash = hash_recovery_code(&plain)?;
        codes.push(RecoveryCode { plain, hash });
    }
    Ok(codes)
}

/// 使用 Argon2 哈希恢复码。
pub fn hash_recovery_code(code: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(code.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| AppError::internal("failed to hash recovery code"))
}

/// 校验恢复码与已存哈希是否匹配。
pub fn verify_recovery_code(code: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|_| AppError::internal("invalid recovery code hash"))?;
    Ok(Argon2::default()
        .verify_password(code.as_bytes(), &parsed)
        .is_ok())
}

/// 生成随机令牌（URL 友好）。
pub fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(40)
        .map(char::from)
        .collect()
}

/// 计算令牌哈希（SHA-256 + HEX）。
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// 使用 Argon2 哈希密码。
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| AppError::internal("failed to hash password"))
}

/// 校验密码哈希。
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|_| AppError::internal("invalid password hash"))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// 生成 TOTP 密钥并返回原始字节与 otpauth URL。
pub fn generate_totp(secret_label: &str, account: &str) -> Result<(Vec<u8>, String), AppError> {
    let secret = Secret::generate_secret();
    let bytes = secret
        .to_bytes()
        .map_err(|_| AppError::internal("failed to generate TOTP secret"))?;
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        bytes.clone(),
        Some(secret_label.to_string()),
        account.to_string(),
    )
    .map_err(|_| AppError::internal("failed to build TOTP"))?;
    Ok((bytes, totp.get_url()))
}

/// 使用存储密钥校验 TOTP 验证码。
pub fn verify_totp(secret: &[u8], code: &str) -> Result<bool, AppError> {
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_vec(),
        None,
        "".to_string(),
    )
    .map_err(|_| AppError::internal("failed to build TOTP"))?;
    Ok(totp.check_current(code).unwrap_or(false))
}

/// 使用 AES-256-GCM 加密密钥。
pub fn encrypt_secret(secret: &[u8], key: &[u8]) -> Result<String, AppError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| AppError::internal("invalid secret encryption key"))?;
    let mut nonce = [0u8; SECRET_NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), secret)
        .map_err(|_| AppError::internal("failed to encrypt secret"))?;

    let mut payload = Vec::with_capacity(SECRET_NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(&nonce);
    payload.extend_from_slice(&ciphertext);
    let encoded = base64::engine::general_purpose::STANDARD.encode(payload);
    Ok(format!("{SECRET_PREFIX}{encoded}"))
}

/// 解密已加密的密钥。
pub fn decrypt_secret(encoded: &str, key: &[u8]) -> Result<Vec<u8>, AppError> {
    let payload = encoded
        .trim()
        .strip_prefix(SECRET_PREFIX)
        .ok_or_else(|| AppError::internal("invalid encrypted secret format"))?;
    let payload = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .map_err(|_| AppError::internal("invalid encrypted secret base64"))?;
    if payload.len() < SECRET_NONCE_LEN {
        return Err(AppError::internal("invalid encrypted secret payload"));
    }
    let (nonce, ciphertext) = payload.split_at(SECRET_NONCE_LEN);
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| AppError::internal("invalid secret encryption key"))?;
    cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| AppError::internal("failed to decrypt secret"))
}

/// 生成用于审计的恢复码标识。
pub fn recovery_code_label(user_id: Uuid, code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(user_id.as_bytes());
    hasher.update(code.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_codes_are_unique() {
        let codes = generate_recovery_codes(5).expect("codes");
        let mut seen = std::collections::HashSet::new();
        for code in codes {
            assert!(seen.insert(code.plain));
        }
    }

    #[test]
    fn totp_round_trip() {
        let (secret, _) = generate_totp("Labor Hours Platform", "user@example.com").expect("totp");
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret.clone(),
            None,
            "".to_string(),
        )
        .expect("build");
        let code = totp.generate_current().expect("code");
        assert!(verify_totp(&secret, &code).expect("verify"));
    }

    #[test]
    fn secret_encrypt_round_trip() {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        let data = b"secret";
        let enc = encrypt_secret(data, &key).expect("encrypt");
        let dec = decrypt_secret(&enc, &key).expect("decrypt");
        assert_eq!(dec, data);
    }

    #[test]
    fn session_token_hash_changes() {
        let token = generate_session_token();
        let hash = hash_session_token(&token);
        assert_ne!(token, hash);
    }
}
