//! VolunteerHours 服务端配置加载。

use std::env;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use base64::Engine;
use url::Url;

use crate::error::AppError;

/// 服务端运行时配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// HTTP 服务绑定地址。
    pub bind_addr: String,
    /// 是否启用开发者模式。
    pub developer_mode: bool,
    /// 是否允许 HTTP（反向代理终止 HTTPS 时使用）。
    pub allow_http: bool,
    /// 数据库连接串。
    pub database_url: String,
    /// WebAuthn 依赖方 ID。
    pub rp_id: String,
    /// WebAuthn 依赖方源 URL。
    pub rp_origin: Url,
    /// TLS 证书路径。
    pub tls_cert_path: PathBuf,
    /// TLS 私钥加密文件路径。
    pub tls_key_path: PathBuf,
    /// 可选：导入 PEM 证书以替换现有证书。
    pub tls_import_cert_path: Option<PathBuf>,
    /// 可选：导入 PEM 私钥以替换现有私钥。
    pub tls_import_key_path: Option<PathBuf>,
    /// 用于 TLS 私钥加密的 Base64 AES-256 密钥。
    pub tls_key_enc_key: Vec<u8>,
    /// 附件与签名的基础目录。
    pub upload_dir: PathBuf,
    /// 会话 Cookie 名称。
    pub session_cookie_name: String,
    /// 会话有效期（秒）。
    pub session_ttl_seconds: i64,
    /// 应用密钥（TOTP、恢复码等）的 Base64 AES-256 密钥。
    pub auth_secret_key: Vec<u8>,
    /// 可选：用于创建初始管理员的引导令牌。
    pub bootstrap_token: Option<String>,
}

impl Config {
    /// 从环境变量加载配置。
    pub fn from_env() -> Result<Self, AppError> {
        let developer_mode = env_bool("DEVELOPER_MODE").unwrap_or(false);
        let allow_http = env_bool("ALLOW_HTTP").unwrap_or(developer_mode);

        let bind_addr = if developer_mode {
            "0.0.0.0:8443".to_string()
        } else {
            env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8443".to_string())
        };
        let database_url = if developer_mode {
            "sqlite://data/dev.db?mode=rwc".to_string()
        } else {
            env::var("DATABASE_URL")
                .map_err(|_| AppError::config("DATABASE_URL is required"))?
        };
        let rp_id = if developer_mode {
            "localhost".to_string()
        } else {
            env::var("RP_ID").map_err(|_| AppError::config("RP_ID is required"))?
        };
        let default_origin = if allow_http {
            "http://localhost:8443"
        } else {
            "https://localhost:8443"
        };
        let rp_origin = if developer_mode {
            default_origin
                .parse::<Url>()
                .map_err(|_| AppError::config("RP_ORIGIN must be a valid URL"))?
        } else {
            env::var("RP_ORIGIN")
                .map_err(|_| AppError::config("RP_ORIGIN is required"))?
                .parse::<Url>()
                .map_err(|_| AppError::config("RP_ORIGIN must be a valid URL"))?
        };
        let tls_cert_path = env::var("TLS_CERT_PATH")
            .unwrap_or_else(|_| "data/tls/cert.pem".to_string())
            .into();
        let tls_key_path = env::var("TLS_KEY_PATH")
            .unwrap_or_else(|_| "data/tls/key.enc".to_string())
            .into();
        let tls_import_cert_path = env::var("TLS_IMPORT_CERT_PEM").ok().map(PathBuf::from);
        let tls_import_key_path = env::var("TLS_IMPORT_KEY_PEM").ok().map(PathBuf::from);
        let tls_key_enc_key = if developer_mode {
            "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=".to_string()
        } else {
            env::var("TLS_KEY_ENC_KEY")
                .map_err(|_| AppError::config("TLS_KEY_ENC_KEY is required"))?
        };
        let tls_key_enc_key = base64::engine::general_purpose::STANDARD
            .decode(tls_key_enc_key)
            .map_err(|_| AppError::config("TLS_KEY_ENC_KEY must be base64"))?;
        if tls_key_enc_key.len() != 32 {
            return Err(AppError::config(
                "TLS_KEY_ENC_KEY must be 32 bytes after base64 decode",
            ));
        }
        let upload_dir = env::var("UPLOAD_DIR")
            .unwrap_or_else(|_| "data/uploads".to_string())
            .into();
        let session_cookie_name = env::var("SESSION_COOKIE_NAME")
            .unwrap_or_else(|_| "vh_session".to_string());
        let session_ttl_seconds = env::var("SESSION_TTL_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<i64>()
            .map_err(|_| AppError::config("SESSION_TTL_SECONDS must be integer"))?;
        let auth_secret_key = if developer_mode {
            "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=".to_string()
        } else {
            env::var("AUTH_SECRET_KEY")
                .map_err(|_| AppError::config("AUTH_SECRET_KEY is required"))?
        };
        let auth_secret_key = base64::engine::general_purpose::STANDARD
            .decode(auth_secret_key)
            .map_err(|_| AppError::config("AUTH_SECRET_KEY must be base64"))?;
        if auth_secret_key.len() != 32 {
            return Err(AppError::config(
                "AUTH_SECRET_KEY must be 32 bytes after base64 decode",
            ));
        }
        let bootstrap_token = if developer_mode {
            None
        } else {
            env::var("BOOTSTRAP_TOKEN").ok()
        };

        Ok(Self {
            bind_addr,
            developer_mode,
            allow_http,
            database_url,
            rp_id,
            rp_origin,
            tls_cert_path,
            tls_key_path,
            tls_import_cert_path,
            tls_import_key_path,
            tls_key_enc_key,
            upload_dir,
            session_cookie_name,
            session_ttl_seconds,
            auth_secret_key,
            bootstrap_token,
        })
    }
}

fn env_bool(key: &str) -> Option<bool> {
    env::var(key).ok().map(|value| {
        matches!(
            value.to_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}
