//! Labor Hours Platform 服务端配置加载。

use std::{env, fs};
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
    /// 前端基础 URL（用于邀请与重置链接）。
    pub base_url: Option<Url>,
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
    /// 邮件发送配置。
    pub mail: Option<MailConfig>,
    /// 学生密码策略。
    pub password_policy: PasswordPolicy,
    /// 重置凭证交付方式（email/code）。
    pub reset_delivery: ResetDelivery,
}

/// 重置凭证交付方式。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResetDelivery {
    Email,
    Code,
}

impl Default for ResetDelivery {
    fn default() -> Self {
        Self::Email
    }
}

/// 邮件发送配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailConfig {
    /// SMTP 服务器地址。
    pub smtp_host: String,
    /// SMTP 端口。
    pub smtp_port: u16,
    /// SMTP 用户名。
    pub smtp_username: String,
    /// SMTP 密码或授权码。
    pub smtp_password: String,
    /// 发件人邮箱地址。
    pub from_address: String,
    /// 发件人显示名称（可选）。
    pub from_name: Option<String>,
    /// 是否启用 TLS。
    pub use_tls: bool,
}

/// 学生密码策略。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    /// 最小长度。
    pub min_length: usize,
    /// 是否要求大写字母。
    pub require_uppercase: bool,
    /// 是否要求小写字母。
    pub require_lowercase: bool,
    /// 是否要求数字。
    pub require_digit: bool,
    /// 是否要求特殊符号。
    pub require_symbol: bool,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: false,
            require_lowercase: false,
            require_digit: true,
            require_symbol: false,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    developer_mode: Option<bool>,
    allow_http: Option<bool>,
    bind_addr: Option<String>,
    database_url: Option<String>,
    rp_id: Option<String>,
    rp_origin: Option<String>,
    base_url: Option<String>,
    tls_cert_path: Option<PathBuf>,
    tls_key_path: Option<PathBuf>,
    tls_import_cert_path: Option<PathBuf>,
    tls_import_key_path: Option<PathBuf>,
    upload_dir: Option<PathBuf>,
    session_cookie_name: Option<String>,
    session_ttl_seconds: Option<i64>,
    mail: Option<MailConfig>,
    password_policy: Option<PasswordPolicyFile>,
    reset_delivery: Option<ResetDelivery>,
}

#[derive(Debug, Deserialize)]
struct PasswordPolicyFile {
    min_length: Option<usize>,
    require_uppercase: Option<bool>,
    require_lowercase: Option<bool>,
    require_digit: Option<bool>,
    require_symbol: Option<bool>,
}

impl Config {
    /// 从环境变量加载配置。
    pub fn from_env() -> Result<Self, AppError> {
        let file = load_config_file()?;
        let file_ref = file.as_ref();
        let developer_mode = env_bool("DEVELOPER_MODE")
            .or_else(|| file_ref.and_then(|cfg| cfg.developer_mode))
            .unwrap_or(false);
        let allow_http = env_bool("ALLOW_HTTP")
            .or_else(|| file_ref.and_then(|cfg| cfg.allow_http))
            .unwrap_or(developer_mode);

        let bind_addr = if developer_mode {
            "0.0.0.0:8443".to_string()
        } else {
            env::var("BIND_ADDR")
                .ok()
                .or_else(|| file_ref.and_then(|cfg| cfg.bind_addr.clone()))
                .unwrap_or_else(|| "0.0.0.0:8443".to_string())
        };
        let database_url = if developer_mode {
            "sqlite://data/dev.db?mode=rwc".to_string()
        } else {
            env::var("DATABASE_URL")
                .ok()
                .or_else(|| file_ref.and_then(|cfg| cfg.database_url.clone()))
                .ok_or_else(|| AppError::config("DATABASE_URL is required"))?
        };
        let rp_id = if developer_mode {
            "localhost".to_string()
        } else {
            env::var("RP_ID")
                .ok()
                .or_else(|| file_ref.and_then(|cfg| cfg.rp_id.clone()))
                .ok_or_else(|| AppError::config("RP_ID is required"))?
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
            let origin = env::var("RP_ORIGIN")
                .ok()
                .or_else(|| file_ref.and_then(|cfg| cfg.rp_origin.clone()))
                .ok_or_else(|| AppError::config("RP_ORIGIN is required"))?;
            origin
                .parse::<Url>()
                .map_err(|_| AppError::config("RP_ORIGIN must be a valid URL"))?
        };
        let base_url = env::var("BASE_URL")
            .ok()
            .or_else(|| file_ref.and_then(|cfg| cfg.base_url.clone()))
            .map(|value| {
                value
                    .parse::<Url>()
                    .map_err(|_| AppError::config("BASE_URL must be a valid URL"))
            })
            .transpose()?;
        let tls_cert_path = env::var("TLS_CERT_PATH")
            .ok()
            .or_else(|| file_ref.and_then(|cfg| cfg.tls_cert_path.clone()).map(|path| path.to_string_lossy().to_string()))
            .unwrap_or_else(|| "data/tls/cert.pem".to_string())
            .into();
        let tls_key_path = env::var("TLS_KEY_PATH")
            .ok()
            .or_else(|| file_ref.and_then(|cfg| cfg.tls_key_path.clone()).map(|path| path.to_string_lossy().to_string()))
            .unwrap_or_else(|| "data/tls/key.enc".to_string())
            .into();
        let tls_import_cert_path = env::var("TLS_IMPORT_CERT_PEM")
            .ok()
            .map(PathBuf::from)
            .or_else(|| file_ref.and_then(|cfg| cfg.tls_import_cert_path.clone()));
        let tls_import_key_path = env::var("TLS_IMPORT_KEY_PEM")
            .ok()
            .map(PathBuf::from)
            .or_else(|| file_ref.and_then(|cfg| cfg.tls_import_key_path.clone()));
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
            .ok()
            .or_else(|| file_ref.and_then(|cfg| cfg.upload_dir.clone()).map(|path| path.to_string_lossy().to_string()))
            .unwrap_or_else(|| "data/uploads".to_string())
            .into();
        let session_cookie_name = env::var("SESSION_COOKIE_NAME")
            .ok()
            .or_else(|| file_ref.and_then(|cfg| cfg.session_cookie_name.clone()))
            .unwrap_or_else(|| "vh_session".to_string());
        let session_ttl_seconds = env::var("SESSION_TTL_SECONDS")
            .ok()
            .or_else(|| file_ref.and_then(|cfg| cfg.session_ttl_seconds.map(|value| value.to_string())))
            .unwrap_or_else(|| "3600".to_string())
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
        let mail = load_mail_config(file_ref)?;
        let password_policy = load_password_policy(file_ref);
        let reset_delivery = env::var("RESET_DELIVERY")
            .ok()
            .and_then(|value| parse_reset_delivery(&value))
            .or_else(|| file_ref.and_then(|cfg| cfg.reset_delivery.clone()))
            .unwrap_or_default();

        Ok(Self {
            bind_addr,
            developer_mode,
            allow_http,
            database_url,
            rp_id,
            rp_origin,
            base_url,
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
            mail,
            password_policy,
            reset_delivery,
        })
    }
}

fn parse_reset_delivery(value: &str) -> Option<ResetDelivery> {
    match value.to_lowercase().as_str() {
        "email" => Some(ResetDelivery::Email),
        "code" => Some(ResetDelivery::Code),
        _ => None,
    }
}

fn load_config_file() -> Result<Option<ConfigFile>, AppError> {
    let explicit_path = env::var("CONFIG_PATH").ok().map(PathBuf::from);
    let path = explicit_path.clone().unwrap_or_else(|| PathBuf::from("config.toml"));
    if !path.exists() {
        if explicit_path.is_some() {
            return Err(AppError::config("CONFIG_PATH not found"));
        }
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .map_err(|_| AppError::config("failed to read config file"))?;
    let config = toml::from_str::<ConfigFile>(&content)
        .map_err(|_| AppError::config("invalid config file"))?;
    Ok(Some(config))
}

fn load_mail_config(file: Option<&ConfigFile>) -> Result<Option<MailConfig>, AppError> {
    let host = env::var("SMTP_HOST").ok().or_else(|| {
        file.and_then(|cfg| cfg.mail.as_ref().map(|mail| mail.smtp_host.clone()))
    });
    let port = env::var("SMTP_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .or_else(|| file.and_then(|cfg| cfg.mail.as_ref().map(|mail| mail.smtp_port)));
    let username = env::var("SMTP_USERNAME").ok().or_else(|| {
        file.and_then(|cfg| cfg.mail.as_ref().map(|mail| mail.smtp_username.clone()))
    });
    let password = env::var("SMTP_PASSWORD").ok().or_else(|| {
        file.and_then(|cfg| cfg.mail.as_ref().map(|mail| mail.smtp_password.clone()))
    });
    let from_address = env::var("SMTP_FROM_ADDRESS").ok().or_else(|| {
        file.and_then(|cfg| cfg.mail.as_ref().map(|mail| mail.from_address.clone()))
    });
    let from_name = env::var("SMTP_FROM_NAME").ok().or_else(|| {
        file.and_then(|cfg| cfg.mail.as_ref().and_then(|mail| mail.from_name.clone()))
    });
    let use_tls = env_bool("SMTP_USE_TLS")
        .or_else(|| file.and_then(|cfg| cfg.mail.as_ref().map(|mail| mail.use_tls)))
        .unwrap_or(true);

    if host.is_none()
        && username.is_none()
        && password.is_none()
        && from_address.is_none()
        && port.is_none()
    {
        return Ok(None);
    }

    let smtp_host = host.ok_or_else(|| AppError::config("SMTP_HOST is required"))?;
    let smtp_port = port.ok_or_else(|| AppError::config("SMTP_PORT is required"))?;
    let smtp_username = username.ok_or_else(|| AppError::config("SMTP_USERNAME is required"))?;
    let smtp_password = password.ok_or_else(|| AppError::config("SMTP_PASSWORD is required"))?;
    let from_address = from_address.ok_or_else(|| AppError::config("SMTP_FROM_ADDRESS is required"))?;

    Ok(Some(MailConfig {
        smtp_host,
        smtp_port,
        smtp_username,
        smtp_password,
        from_address,
        from_name,
        use_tls,
    }))
}

fn load_password_policy(file: Option<&ConfigFile>) -> PasswordPolicy {
    let mut policy = PasswordPolicy::default();
    if let Some(file_policy) = file.and_then(|cfg| cfg.password_policy.as_ref()) {
        if let Some(value) = file_policy.min_length {
            policy.min_length = value;
        }
        if let Some(value) = file_policy.require_uppercase {
            policy.require_uppercase = value;
        }
        if let Some(value) = file_policy.require_lowercase {
            policy.require_lowercase = value;
        }
        if let Some(value) = file_policy.require_digit {
            policy.require_digit = value;
        }
        if let Some(value) = file_policy.require_symbol {
            policy.require_symbol = value;
        }
    }
    policy
}

fn env_bool(key: &str) -> Option<bool> {
    env::var(key).ok().map(|value| {
        matches!(
            value.to_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}
