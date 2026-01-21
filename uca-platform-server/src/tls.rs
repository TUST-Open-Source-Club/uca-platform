//! TLS 证书管理与私钥加密。

use std::fs;
use std::io::Write;
use std::path::Path;

use aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::Engine;
use rand::{rngs::OsRng, RngCore};
use rcgen::generate_simple_self_signed;

use crate::config::Config;
use crate::error::AppError;

const ENC_PREFIX: &str = "ENCv1:";
const NONCE_LEN: usize = 12;

/// 确保 TLS 证书与加密私钥存在，可选导入外部 PEM 文件。
pub fn ensure_tls_material(config: &Config) -> Result<(), AppError> {
    if let (Some(cert_path), Some(key_path)) = (
        config.tls_import_cert_path.as_ref(),
        config.tls_import_key_path.as_ref(),
    ) {
        let cert_pem = fs::read(cert_path)
            .map_err(|err| AppError::internal(&format!("failed to read cert: {err}")))?;
        let key_pem = fs::read(key_path)
            .map_err(|err| AppError::internal(&format!("failed to read key: {err}")))?;
        write_cert_and_key(
            &config.tls_cert_path,
            &config.tls_key_path,
            &cert_pem,
            &key_pem,
            &config.tls_key_enc_key,
        )?;
        return Ok(());
    }

    if config.tls_cert_path.exists() && config.tls_key_path.exists() {
        return Ok(());
    }

    let certified = generate_self_signed(&config.rp_id)?;
    let cert_pem = certified.cert.pem();
    let key_pem = certified.key_pair.serialize_pem();

    write_cert_and_key(
        &config.tls_cert_path,
        &config.tls_key_path,
        cert_pem.as_bytes(),
        key_pem.as_bytes(),
        &config.tls_key_enc_key,
    )?;

    Ok(())
}

/// 读取解密后的证书与私钥 PEM 内容。
pub fn load_tls_pem(config: &Config) -> Result<(Vec<u8>, Vec<u8>), AppError> {
    let cert_pem = fs::read(&config.tls_cert_path)
        .map_err(|err| AppError::internal(&format!("failed to read cert: {err}")))?;
    let key_enc = fs::read_to_string(&config.tls_key_path)
        .map_err(|err| AppError::internal(&format!("failed to read key: {err}")))?;
    let key_pem = decrypt_private_key(&key_enc, &config.tls_key_enc_key)?;

    Ok((cert_pem, key_pem))
}

fn generate_self_signed(rp_id: &str) -> Result<rcgen::CertifiedKey, AppError> {
    generate_simple_self_signed(vec![rp_id.to_string()])
        .map_err(|err| AppError::internal(&format!("failed to build certificate: {err}")))
}

fn write_cert_and_key(
    cert_path: &Path,
    key_path: &Path,
    cert_pem: &[u8],
    key_pem: &[u8],
    enc_key: &[u8],
) -> Result<(), AppError> {
    create_parent_dir(cert_path)?;
    create_parent_dir(key_path)?;

    fs::write(cert_path, cert_pem)
        .map_err(|err| AppError::internal(&format!("failed to write cert: {err}")))?;
    let encrypted = encrypt_private_key(key_pem, enc_key)?;
    let mut file = fs::File::create(key_path)
        .map_err(|err| AppError::internal(&format!("failed to write key: {err}")))?;
    file.write_all(encrypted.as_bytes())
        .map_err(|err| AppError::internal(&format!("failed to write key: {err}")))?;
    Ok(())
}

fn create_parent_dir(path: &Path) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| AppError::internal(&format!("failed to create dir: {err}")))?;
    }
    Ok(())
}

fn encrypt_private_key(key_pem: &[u8], enc_key: &[u8]) -> Result<String, AppError> {
    let cipher = Aes256Gcm::new_from_slice(enc_key)
        .map_err(|_| AppError::internal("invalid TLS encryption key"))?;
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), key_pem)
        .map_err(|_| AppError::internal("failed to encrypt private key"))?;

    let mut payload = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(&nonce);
    payload.extend_from_slice(&ciphertext);
    let encoded = base64::engine::general_purpose::STANDARD.encode(payload);
    Ok(format!("{ENC_PREFIX}{encoded}"))
}

fn decrypt_private_key(encoded: &str, enc_key: &[u8]) -> Result<Vec<u8>, AppError> {
    let payload = encoded
        .trim()
        .strip_prefix(ENC_PREFIX)
        .ok_or_else(|| AppError::internal("invalid encrypted key format"))?;
    let payload = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .map_err(|_| AppError::internal("invalid encrypted key base64"))?;
    if payload.len() < NONCE_LEN {
        return Err(AppError::internal("invalid encrypted key payload"));
    }
    let (nonce, ciphertext) = payload.split_at(NONCE_LEN);
    let cipher = Aes256Gcm::new_from_slice(enc_key)
        .map_err(|_| AppError::internal("invalid TLS encryption key"))?;
    cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| AppError::internal("failed to decrypt private key"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_round_trip() {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        let pem = b"TEST-KEY";

        let encrypted = encrypt_private_key(pem, &key).expect("encrypt");
        let decrypted = decrypt_private_key(&encrypted, &key).expect("decrypt");

        assert_eq!(decrypted, pem);
    }
}
