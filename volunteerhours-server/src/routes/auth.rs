//! 认证处理器（Passkey、TOTP、恢复码）。

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use base64::Engine;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set, ActiveModelTrait, PaginatorTrait};
use serde::{Deserialize, Serialize};
use chrono::{Duration as ChronoDuration, Utc};
use time::{Duration as TimeDuration, OffsetDateTime};
use uuid::Uuid;
use webauthn_rs::prelude::{CreationChallengeResponse, PublicKeyCredential, RegisterPublicKeyCredential};

use crate::{
    auth::{
        decrypt_secret, encrypt_secret, generate_recovery_codes, generate_session_token,
        generate_totp, hash_session_token, verify_recovery_code, verify_totp,
    },
    entities::{
        devices, passkeys, recovery_codes, sessions, totp_secrets, users, Device, Passkey,
        RecoveryCode, Session, TotpSecret, User,
    },
    error::AppError,
    state::{AppState, PasskeyAuthSession, PasskeyRegisterSession},
};

/// 基础健康检查响应。
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// 健康状态。
    pub status: String,
}

/// 健康检查接口。
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

/// 当前登录用户信息响应。
#[derive(Debug, Serialize)]
pub struct CurrentUserResponse {
    /// 用户 ID。
    pub id: Uuid,
    /// 用户名。
    pub username: String,
    /// 展示名。
    pub display_name: String,
    /// 角色。
    pub role: String,
}

/// 获取当前会话的用户信息。
pub async fn current_user(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<CurrentUserResponse>, AppError> {
    let user = require_session(&state, &jar).await?;
    Ok(Json(CurrentUserResponse {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        role: user.role,
    }))
}

/// 引导创建管理员的请求体。
#[derive(Debug, Deserialize)]
pub struct BootstrapRequest {
    /// 引导令牌（如已配置）。
    pub token: Option<String>,
    /// 管理员用户名。
    pub username: String,
    /// 管理员展示名。
    pub display_name: String,
}

/// 引导创建管理员响应。
#[derive(Debug, Serialize)]
pub struct BootstrapResponse {
    /// 新建用户 ID。
    pub user_id: Uuid,
}

/// 创建初始管理员用户（仅在无用户时允许）。
pub async fn bootstrap_admin(
    State(state): State<AppState>,
    Json(payload): Json<BootstrapRequest>,
) -> Result<Json<BootstrapResponse>, AppError> {
    if let Some(expected) = state.config.bootstrap_token.as_ref() {
        if payload.token.as_deref() != Some(expected.as_str()) {
            return Err(AppError::auth("invalid bootstrap token"));
        }
    }

    let existing = User::find().count(&state.db).await
        .map_err(|err| AppError::Database(err.to_string()))?;
    if existing > 0 {
        return Err(AppError::bad_request("bootstrap already completed"));
    }

    let now = Utc::now();
    let id = Uuid::new_v4();
    let user = users::ActiveModel {
        id: Set(id),
        username: Set(payload.username),
        display_name: Set(payload.display_name),
        role: Set("admin".to_string()),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    };
    user.insert(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(BootstrapResponse { user_id: id }))
}

/// 开始 Passkey 注册的请求体。
#[derive(Debug, Deserialize)]
pub struct PasskeyRegisterStartRequest {
    /// 注册 Passkey 的用户名。
    pub username: String,
}

/// Passkey 注册开始响应。
#[derive(Debug, Serialize)]
pub struct PasskeyRegisterStartResponse {
    /// 服务端会话 ID。
    pub session_id: Uuid,
    /// Passkey 挑战。
    pub public_key: CreationChallengeResponse,
}

/// 开始 Passkey 注册。
pub async fn passkey_register_start(
    State(state): State<AppState>,
    Json(payload): Json<PasskeyRegisterStartRequest>,
) -> Result<Json<PasskeyRegisterStartResponse>, AppError> {
    let user = User::find()
        .filter(users::Column::Username.eq(payload.username))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("user not found"))?;

    let existing_passkeys = Passkey::find()
        .filter(passkeys::Column::UserId.eq(user.id))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let exclude_credentials = existing_passkeys
        .iter()
        .filter_map(|record| {
            base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(&record.credential_id)
                .ok()
        })
        .map(webauthn_rs::prelude::CredentialID::from)
        .collect::<Vec<_>>();

    let (challenge, reg_state) = state
        .webauthn
        .start_passkey_registration(
            user.id,
            &user.username,
            &user.display_name,
            Some(exclude_credentials),
        )
        .map_err(|err| AppError::internal(&format!("passkey registration start failed: {err}")))?;

    let session_id = Uuid::new_v4();
    let session = PasskeyRegisterSession {
        user_id: user.id,
        state: reg_state,
        created_at: OffsetDateTime::now_utc(),
    };

    state
        .passkey_state
        .lock()
        .await
        .insert_register(session_id, session);

    Ok(Json(PasskeyRegisterStartResponse {
        session_id,
        public_key: challenge,
    }))
}

/// 完成 Passkey 注册的请求体。
#[derive(Debug, Deserialize)]
pub struct PasskeyRegisterFinishRequest {
    /// 开始注册时的会话 ID。
    pub session_id: Uuid,
    /// 浏览器返回的凭据。
    pub credential: RegisterPublicKeyCredential,
    /// 设备标签（用于管理界面）。
    pub device_label: Option<String>,
}

/// Passkey 注册完成响应。
#[derive(Debug, Serialize)]
pub struct PasskeyRegisterFinishResponse {
    /// Passkey 记录 ID。
    pub passkey_id: Uuid,
}

/// 完成 Passkey 注册并保存凭据。
pub async fn passkey_register_finish(
    State(state): State<AppState>,
    Json(payload): Json<PasskeyRegisterFinishRequest>,
) -> Result<Json<PasskeyRegisterFinishResponse>, AppError> {
    let session = state
        .passkey_state
        .lock()
        .await
        .take_register(&payload.session_id)
        .ok_or_else(|| AppError::bad_request("invalid or expired session"))?;

    let passkey = state
        .webauthn
        .finish_passkey_registration(&payload.credential, &session.state)
        .map_err(|err| AppError::internal(&format!("passkey registration failed: {err}")))?;

    let cred_id = passkey.cred_id();
    let cred_id_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(cred_id.as_ref());

    let existing = Passkey::find()
        .filter(passkeys::Column::CredentialId.eq(&cred_id_b64))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    if existing.is_some() {
        return Err(AppError::bad_request("credential already registered"));
    }

    let now = Utc::now();
    let passkey_id = Uuid::new_v4();
    let passkey_json = serde_json::to_string(&passkey)
        .map_err(|_| AppError::internal("failed to serialize passkey"))?;

    passkeys::ActiveModel {
        id: Set(passkey_id),
        user_id: Set(session.user_id),
        credential_id: Set(cred_id_b64.clone()),
        passkey_json: Set(passkey_json),
        created_at: Set(now),
        last_used_at: Set(None),
    }
    .insert(&state.db)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    devices::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(session.user_id),
        device_type: Set("passkey".to_string()),
        label: Set(payload
            .device_label
            .unwrap_or_else(|| format!("Passkey-{passkey_id}"))),
        credential_id: Set(Some(cred_id_b64)),
        created_at: Set(now),
        last_used_at: Set(None),
    }
    .insert(&state.db)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(PasskeyRegisterFinishResponse { passkey_id }))
}

/// 开始 Passkey 认证的请求体。
#[derive(Debug, Deserialize)]
pub struct PasskeyLoginStartRequest {
    /// 待认证用户名。
    pub username: String,
}

/// Passkey 认证开始响应。
#[derive(Debug, Serialize)]
pub struct PasskeyLoginStartResponse {
    /// 服务端会话 ID。
    pub session_id: Uuid,
    /// Passkey 挑战。
    pub public_key: webauthn_rs::prelude::RequestChallengeResponse,
}

/// 开始 Passkey 认证。
pub async fn passkey_login_start(
    State(state): State<AppState>,
    Json(payload): Json<PasskeyLoginStartRequest>,
) -> Result<Json<PasskeyLoginStartResponse>, AppError> {
    let user = User::find()
        .filter(users::Column::Username.eq(payload.username))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("user not found"))?;

    let passkey_records = Passkey::find()
        .filter(passkeys::Column::UserId.eq(user.id))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    if passkey_records.is_empty() {
        return Err(AppError::bad_request("no passkeys registered"));
    }

    let passkeys = passkey_records
        .iter()
        .map(|record| serde_json::from_str(&record.passkey_json))
        .collect::<Result<Vec<webauthn_rs::prelude::Passkey>, _>>()
        .map_err(|_| AppError::internal("failed to parse passkey"))?;

    let (challenge, auth_state) = state
        .webauthn
        .start_passkey_authentication(&passkeys)
        .map_err(|err| AppError::internal(&format!("passkey login start failed: {err}")))?;

    let session_id = Uuid::new_v4();
    let session = PasskeyAuthSession {
        user_id: user.id,
        state: auth_state,
        created_at: OffsetDateTime::now_utc(),
    };

    state
        .passkey_state
        .lock()
        .await
        .insert_auth(session_id, session);

    Ok(Json(PasskeyLoginStartResponse {
        session_id,
        public_key: challenge,
    }))
}

/// 完成 Passkey 认证的请求体。
#[derive(Debug, Deserialize)]
pub struct PasskeyLoginFinishRequest {
    /// 开始认证时的会话 ID。
    pub session_id: Uuid,
    /// 浏览器返回的凭据。
    pub credential: PublicKeyCredential,
}

/// Passkey 认证完成响应。
#[derive(Debug, Serialize)]
pub struct PasskeyLoginFinishResponse {
    /// 会话对应的用户 ID。
    pub user_id: Uuid,
}

/// 完成 Passkey 认证，更新计数并创建会话 Cookie。
pub async fn passkey_login_finish(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<PasskeyLoginFinishRequest>,
) -> Result<impl IntoResponse, AppError> {
    let session = state
        .passkey_state
        .lock()
        .await
        .take_auth(&payload.session_id)
        .ok_or_else(|| AppError::bad_request("invalid or expired session"))?;

    let auth_result = state
        .webauthn
        .finish_passkey_authentication(&payload.credential, &session.state)
        .map_err(|err| AppError::auth(&format!("passkey login failed: {err}")))?;

    let cred_id_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(auth_result.cred_id().as_ref());

    let record = Passkey::find()
        .filter(passkeys::Column::CredentialId.eq(&cred_id_b64))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::auth("credential not found"))?;

    let mut stored_passkey: webauthn_rs::prelude::Passkey = serde_json::from_str(&record.passkey_json)
        .map_err(|_| AppError::internal("failed to parse passkey"))?;
    if stored_passkey.update_credential(&auth_result).unwrap_or(false) {
        let updated_json = serde_json::to_string(&stored_passkey)
            .map_err(|_| AppError::internal("failed to serialize passkey"))?;
        let mut active: passkeys::ActiveModel = record.into();
        active.passkey_json = Set(updated_json);
        active.last_used_at = Set(Some(Utc::now()));
        active
            .update(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    }

    let device = devices::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(session.user_id),
        device_type: Set("passkey".to_string()),
        label: Set("Passkey".to_string()),
        credential_id: Set(Some(cred_id_b64.clone())),
        created_at: Set(Utc::now()),
        last_used_at: Set(Some(Utc::now())),
    };

    if let Some(existing) = Device::find()
        .filter(devices::Column::UserId.eq(session.user_id))
        .filter(devices::Column::CredentialId.eq(Some(cred_id_b64)))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
    {
        let mut active: devices::ActiveModel = existing.into();
        active.last_used_at = Set(Some(Utc::now()));
        active
            .update(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    } else {
        device
            .insert(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    }

    let (jar, user_id) = create_session_cookie(&state, jar, session.user_id).await?;

    Ok((jar, Json(PasskeyLoginFinishResponse { user_id })))
}

/// 开始 TOTP 绑定的请求体。
#[derive(Debug, Deserialize)]
pub struct TotpEnrollStartRequest {
    /// 可选设备标签。
    pub device_label: Option<String>,
}

/// TOTP 绑定开始响应。
#[derive(Debug, Serialize)]
pub struct TotpEnrollStartResponse {
    /// 绑定流程 ID。
    pub enrollment_id: Uuid,
    /// 展示给用户的 otpauth URL。
    pub otpauth_url: String,
}

/// 为当前用户开始 TOTP 绑定。
pub async fn totp_enroll_start(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<TotpEnrollStartRequest>,
) -> Result<Json<TotpEnrollStartResponse>, AppError> {
    let user = require_session(&state, &jar).await?;

    let (secret, url) = generate_totp("VolunteerHours", &user.username)?;
    let encrypted = encrypt_secret(&secret, &state.config.auth_secret_key)?;

    let now = Utc::now();
    let enrollment_id = Uuid::new_v4();

    totp_secrets::ActiveModel {
        id: Set(enrollment_id),
        user_id: Set(user.id),
        secret_enc: Set(encrypted),
        enabled: Set(false),
        created_at: Set(now),
        verified_at: Set(None),
    }
    .insert(&state.db)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    if let Some(label) = payload.device_label {
        devices::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user.id),
            device_type: Set("totp".to_string()),
            label: Set(label),
            credential_id: Set(None),
            created_at: Set(now),
            last_used_at: Set(None),
        }
        .insert(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    }

    Ok(Json(TotpEnrollStartResponse {
        enrollment_id,
        otpauth_url: url,
    }))
}

/// 完成 TOTP 绑定的请求体。
#[derive(Debug, Deserialize)]
pub struct TotpEnrollFinishRequest {
    /// 绑定开始时的 ID。
    pub enrollment_id: Uuid,
    /// 认证器生成的 TOTP 验证码。
    pub code: String,
}

/// 完成 TOTP 绑定。
pub async fn totp_enroll_finish(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<TotpEnrollFinishRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session(&state, &jar).await?;

    let record = TotpSecret::find_by_id(payload.enrollment_id)
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("enrollment not found"))?;
    if record.user_id != user.id {
        return Err(AppError::auth("forbidden"));
    }

    let secret = decrypt_secret(&record.secret_enc, &state.config.auth_secret_key)?;
    if !verify_totp(&secret, &payload.code)? {
        return Err(AppError::auth("invalid TOTP"));
    }

    let mut active: totp_secrets::ActiveModel = record.into();
    active.enabled = Set(true);
    active.verified_at = Set(Some(Utc::now()));
    active
        .update(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// TOTP 登录验证的请求体。
#[derive(Debug, Deserialize)]
pub struct TotpVerifyRequest {
    /// 用户名。
    pub username: String,
    /// TOTP 验证码。
    pub code: String,
}

/// 校验 TOTP 并创建会话。
pub async fn totp_verify(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<TotpVerifyRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::find()
        .filter(users::Column::Username.eq(payload.username))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("user not found"))?;

    let secret = TotpSecret::find()
        .filter(totp_secrets::Column::UserId.eq(user.id))
        .filter(totp_secrets::Column::Enabled.eq(true))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::bad_request("no TOTP enrolled"))?;

    let raw = decrypt_secret(&secret.secret_enc, &state.config.auth_secret_key)?;
    if !verify_totp(&raw, &payload.code)? {
        return Err(AppError::auth("invalid TOTP"));
    }

    let (jar, user_id) = create_session_cookie(&state, jar, user.id).await?;
    Ok((jar, Json(serde_json::json!({"user_id": user_id}))))
}

/// 恢复码验证的请求体。
#[derive(Debug, Deserialize)]
pub struct RecoveryVerifyRequest {
    /// 用户名。
    pub username: String,
    /// 恢复码。
    pub code: String,
}

/// 校验恢复码并创建会话。
pub async fn recovery_verify(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<RecoveryVerifyRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::find()
        .filter(users::Column::Username.eq(payload.username))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("user not found"))?;

    let codes = RecoveryCode::find()
        .filter(recovery_codes::Column::UserId.eq(user.id))
        .filter(recovery_codes::Column::UsedAt.is_null())
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    for code in codes {
        if verify_recovery_code(&payload.code, &code.code_hash)? {
            let mut active: recovery_codes::ActiveModel = code.into();
            active.used_at = Set(Some(Utc::now()));
            active
                .update(&state.db)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?;

            let (jar, user_id) = create_session_cookie(&state, jar, user.id).await?;
            return Ok((jar, Json(serde_json::json!({"user_id": user_id}))));
        }
    }

    Err(AppError::auth("invalid recovery code"))
}

/// 为当前用户重新生成恢复码。
pub async fn recovery_regenerate(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session(&state, &jar).await?;

    RecoveryCode::delete_many()
        .filter(recovery_codes::Column::UserId.eq(user.id))
        .exec(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let codes = generate_recovery_codes(8)?;
    let now = Utc::now();
    for code in &codes {
        recovery_codes::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user.id),
            code_hash: Set(code.hash.clone()),
            used_at: Set(None),
            created_at: Set(now),
        }
        .insert(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    }

    Ok(Json(serde_json::json!({
        "codes": codes.into_iter().map(|c| c.plain).collect::<Vec<_>>()
    })))
}

/// 列出当前用户的设备。
pub async fn list_devices(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<devices::Model>>, AppError> {
    let user = require_session(&state, &jar).await?;
    let devices = Device::find()
        .filter(devices::Column::UserId.eq(user.id))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(Json(devices))
}

/// 删除当前用户的设备。
pub async fn delete_device(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(device_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session(&state, &jar).await?;
    Device::delete_many()
        .filter(devices::Column::UserId.eq(user.id))
        .filter(devices::Column::Id.eq(device_id))
        .exec(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}

async fn create_session_cookie(
    state: &AppState,
    jar: CookieJar,
    user_id: Uuid,
) -> Result<(CookieJar, Uuid), AppError> {
    let token = generate_session_token();
    let token_hash = hash_session_token(&token);
    let now_db = Utc::now();
    let expires_db = now_db + ChronoDuration::seconds(state.config.session_ttl_seconds);
    let expires_cookie = OffsetDateTime::now_utc()
        + TimeDuration::seconds(state.config.session_ttl_seconds);

    sessions::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        token_hash: Set(token_hash),
        expires_at: Set(expires_db),
        created_at: Set(now_db),
        last_seen_at: Set(Some(now_db)),
    }
    .insert(&state.db)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    let cookie = Cookie::build((state.config.session_cookie_name.clone(), token))
        .http_only(true)
        .secure(!state.config.allow_http)
        .same_site(SameSite::Strict)
        .path("/")
        .expires(expires_cookie)
        .build();

    Ok((jar.add(cookie), user_id))
}

async fn require_session(state: &AppState, jar: &CookieJar) -> Result<users::Model, AppError> {
    let token = jar
        .get(&state.config.session_cookie_name)
        .ok_or_else(|| AppError::auth("missing session"))?
        .value()
        .to_string();
    let token_hash = hash_session_token(&token);
    let now = Utc::now();

    let session = Session::find()
        .filter(sessions::Column::TokenHash.eq(token_hash))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::auth("invalid session"))?;

    if session.expires_at < now {
        return Err(AppError::auth("session expired"));
    }

    let user = User::find_by_id(session.user_id)
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::auth("user not found"))?;

    Ok(user)
}
