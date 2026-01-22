//! 应用共享状态与内存存储。

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
use webauthn_rs::prelude::{PasskeyAuthentication, PasskeyRegistration, Webauthn};

use sea_orm::DatabaseConnection;

use crate::config::Config;
use crate::error::AppError;

/// 认证流程状态的有效期。
const CHALLENGE_TTL_SECONDS: i64 = 300;
/// 二次验证令牌有效期。
const REAUTH_TTL_SECONDS: i64 = 300;

/// 进行中的 Passkey 注册会话。
#[derive(Debug)]
pub struct PasskeyRegisterSession {
    /// 注册用户 ID。
    pub user_id: Uuid,
    /// WebAuthn 注册状态。
    pub state: PasskeyRegistration,
    /// 创建时间，用于过期检查。
    pub created_at: OffsetDateTime,
}

/// 进行中的 Passkey 认证会话。
#[derive(Debug)]
pub struct PasskeyAuthSession {
    /// 认证用户 ID。
    pub user_id: Uuid,
    /// WebAuthn 认证状态。
    pub state: PasskeyAuthentication,
    /// 创建时间，用于过期检查。
    pub created_at: OffsetDateTime,
}

/// Passkey 流程的内存状态存储。
#[derive(Debug, Default)]
pub struct PasskeyStateStore {
    register: HashMap<Uuid, PasskeyRegisterSession>,
    authenticate: HashMap<Uuid, PasskeyAuthSession>,
}

impl PasskeyStateStore {
    /// 写入注册会话。
    pub fn insert_register(&mut self, session_id: Uuid, session: PasskeyRegisterSession) {
        self.register.insert(session_id, session);
    }

    /// 写入认证会话。
    pub fn insert_auth(&mut self, session_id: Uuid, session: PasskeyAuthSession) {
        self.authenticate.insert(session_id, session);
    }

    /// 取出并移除有效的注册会话。
    pub fn take_register(&mut self, session_id: &Uuid) -> Option<PasskeyRegisterSession> {
        self.evict_expired();
        self.register.remove(session_id)
    }

    /// 取出并移除有效的认证会话。
    pub fn take_auth(&mut self, session_id: &Uuid) -> Option<PasskeyAuthSession> {
        self.evict_expired();
        self.authenticate.remove(session_id)
    }

    fn evict_expired(&mut self) {
        let expiry = OffsetDateTime::now_utc() - Duration::seconds(CHALLENGE_TTL_SECONDS);
        self.register
            .retain(|_, session| session.created_at > expiry);
        self.authenticate
            .retain(|_, session| session.created_at > expiry);
    }
}

/// 进行中的 Passkey 重新验证会话。
#[derive(Debug, Default)]
pub struct ReauthPasskeyStore {
    sessions: HashMap<Uuid, PasskeyAuthSession>,
}

impl ReauthPasskeyStore {
    pub fn insert(&mut self, session_id: Uuid, session: PasskeyAuthSession) {
        self.sessions.insert(session_id, session);
    }

    pub fn take(&mut self, session_id: &Uuid) -> Option<PasskeyAuthSession> {
        self.evict_expired();
        self.sessions.remove(session_id)
    }

    fn evict_expired(&mut self) {
        let expiry = OffsetDateTime::now_utc() - Duration::seconds(CHALLENGE_TTL_SECONDS);
        self.sessions
            .retain(|_, session| session.created_at > expiry);
    }
}

/// 二次验证令牌会话。
#[derive(Debug)]
pub struct ReauthSession {
    pub user_id: Uuid,
    pub created_at: OffsetDateTime,
}

/// 二次验证令牌存储。
#[derive(Debug, Default)]
pub struct ReauthStateStore {
    sessions: HashMap<String, ReauthSession>,
}

impl ReauthStateStore {
    pub fn insert(&mut self, token: String, session: ReauthSession) {
        self.sessions.insert(token, session);
    }

    pub fn take(&mut self, token: &str) -> Option<ReauthSession> {
        self.evict_expired();
        self.sessions.remove(token)
    }

    fn evict_expired(&mut self) {
        let expiry = OffsetDateTime::now_utc() - Duration::seconds(REAUTH_TTL_SECONDS);
        self.sessions.retain(|_, session| session.created_at > expiry);
    }
}

/// 应用共享状态。
#[derive(Clone)]
pub struct AppState {
    /// 运行时配置。
    pub config: Arc<Config>,
    /// 数据库连接池。
    pub db: DatabaseConnection,
    /// WebAuthn 实例。
    pub webauthn: Arc<Webauthn>,
    /// Passkey 流程状态。
    pub passkey_state: Arc<Mutex<PasskeyStateStore>>,
    /// Passkey 重新验证状态。
    pub reauth_passkey_state: Arc<Mutex<ReauthPasskeyStore>>,
    /// 二次验证令牌状态。
    pub reauth_state: Arc<Mutex<ReauthStateStore>>,
}

impl AppState {
    /// 创建应用共享状态。
    pub fn new(config: Arc<Config>, db: DatabaseConnection, webauthn: Webauthn) -> Result<Self, AppError> {
        Ok(Self {
            config,
            db,
            webauthn: Arc::new(webauthn),
            passkey_state: Arc::new(Mutex::new(PasskeyStateStore::default())),
            reauth_passkey_state: Arc::new(Mutex::new(ReauthPasskeyStore::default())),
            reauth_state: Arc::new(Mutex::new(ReauthStateStore::default())),
        })
    }
}
