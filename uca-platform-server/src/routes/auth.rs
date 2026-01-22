//! 认证处理器（Passkey、TOTP、恢复码）。

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use base64::Engine;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use sea_orm::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};
use chrono::{Duration as ChronoDuration, Utc};
use time::{Duration as TimeDuration, OffsetDateTime};
use uuid::Uuid;
use webauthn_rs::prelude::{CreationChallengeResponse, PublicKeyCredential, RegisterPublicKeyCredential};

use crate::{
    auth::{
        decrypt_secret, encrypt_secret, generate_recovery_codes, generate_session_token,
        generate_token, generate_totp, hash_password, hash_session_token, hash_token,
        verify_password, verify_recovery_code, verify_totp,
    },
    entities::{
        auth_resets, devices, invites, passkeys, recovery_codes, sessions, totp_secrets, users,
        AuthReset, Device, Invite, Passkey, RecoveryCode, Session, TotpSecret, User,
    },
    error::AppError,
    mailer::send_mail,
    policy::load_password_policy,
    state::{AppState, PasskeyAuthSession, PasskeyRegisterSession},
};

const PASSWORD_RESET_TTL_MINUTES: i64 = 30;

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

/// 引导状态响应。
#[derive(Debug, Serialize)]
pub struct BootstrapStatusResponse {
    /// 是否已完成初始化。
    pub ready: bool,
    /// 是否仍需完成 TOTP 绑定。
    pub needs_totp: bool,
}

/// 登录方式查询参数。
#[derive(Debug, Deserialize)]
pub struct LoginOptionsQuery {
    /// 用户名。
    pub username: String,
}

/// 登录方式响应。
#[derive(Debug, Serialize)]
pub struct LoginOptionsResponse {
    /// 允许的登录方式。
    pub methods: Vec<String>,
}

/// 密码登录请求。
#[derive(Debug, Deserialize)]
pub struct PasswordLoginRequest {
    /// 用户名。
    pub username: String,
    /// 密码。
    pub password: String,
}

/// 绑定邮箱请求。
#[derive(Debug, Deserialize)]
pub struct EmailBindRequest {
    /// 邮箱地址。
    pub email: String,
}

/// 修改密码请求。
#[derive(Debug, Deserialize)]
pub struct PasswordChangeRequest {
    /// 当前密码。
    pub current_password: String,
    /// 新密码。
    pub new_password: String,
}

/// 发起密码重置请求。
#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    /// 用户名。
    pub username: String,
}

/// 完成密码重置请求。
#[derive(Debug, Deserialize)]
pub struct PasswordResetConfirmRequest {
    /// 重置令牌。
    pub token: String,
    /// 新密码。
    pub new_password: String,
}

/// 邀请状态响应。
#[derive(Debug, Serialize)]
pub struct InviteStatusResponse {
    /// 是否有效。
    pub valid: bool,
    /// 邮箱。
    pub email: Option<String>,
    /// 用户名。
    pub username: Option<String>,
    /// 展示名。
    pub display_name: Option<String>,
    /// 角色。
    pub role: Option<String>,
    /// 过期时间。
    pub expires_at: Option<DateTimeUtc>,
}

/// 邀请接受请求。
#[derive(Debug, Deserialize)]
pub struct InviteAcceptRequest {
    /// 邀请令牌。
    pub token: String,
}

/// 邀请接受响应。
#[derive(Debug, Serialize)]
pub struct InviteAcceptResponse {
    /// 新建用户 ID。
    pub user_id: Uuid,
    /// 用户名。
    pub username: String,
    /// 角色。
    pub role: String,
}

/// 重置状态响应。
#[derive(Debug, Serialize)]
pub struct ResetStatusResponse {
    /// 是否有效。
    pub valid: bool,
    /// 目的（totp/passkey/password）。
    pub purpose: Option<String>,
}

/// 重置消费请求。
#[derive(Debug, Deserialize)]
pub struct ResetConsumeRequest {
    /// 重置令牌。
    pub token: String,
}

/// 重置消费响应。
#[derive(Debug, Serialize)]
pub struct ResetConsumeResponse {
    /// 用户 ID。
    pub user_id: Uuid,
    /// 目的。
    pub purpose: String,
}

/// 获取引导状态。
pub async fn bootstrap_status(
    State(state): State<AppState>,
) -> Result<Json<BootstrapStatusResponse>, AppError> {
    let existing = User::find()
        .count(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    let admin_totp = TotpSecret::find()
        .inner_join(users::Entity)
        .filter(totp_secrets::Column::Enabled.eq(true))
        .filter(users::Column::Role.eq("admin"))
        .count(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    let needs_totp = existing > 0 && admin_totp == 0;
    Ok(Json(BootstrapStatusResponse {
        ready: existing > 0 && admin_totp > 0,
        needs_totp,
    }))
}

/// 获取用户允许的登录方式。
pub async fn login_options(
    State(state): State<AppState>,
    Query(query): Query<LoginOptionsQuery>,
) -> Result<Json<LoginOptionsResponse>, AppError> {
    let user = User::find()
        .filter(users::Column::Username.eq(&query.username))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("user not found"))?;

    let mut methods = vec!["passkey".to_string(), "totp".to_string(), "recovery".to_string()];
    if user.role == "student" && user.allow_password_login && user.password_hash.is_some() {
        methods.push("password".to_string());
    }

    Ok(Json(LoginOptionsResponse { methods }))
}

/// 创建初始管理员用户（仅在无用户时允许）。
pub async fn bootstrap_admin(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<BootstrapRequest>,
) -> Result<(CookieJar, Json<BootstrapResponse>), AppError> {
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
        email: Set(None),
        password_hash: Set(None),
        allow_password_login: Set(false),
        password_updated_at: Set(None),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    };
    users::Entity::insert(user)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let (jar, _) = create_session_cookie(&state, jar, id).await?;

    Ok((jar, Json(BootstrapResponse { user_id: id })))
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
    jar: CookieJar,
    Json(payload): Json<PasskeyRegisterStartRequest>,
) -> Result<Json<PasskeyRegisterStartResponse>, AppError> {
    let user = require_session(&state, &jar).await?;
    if user.username != payload.username {
        return Err(AppError::auth("forbidden"));
    }

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

    let passkey_model = passkeys::ActiveModel {
        id: Set(passkey_id),
        user_id: Set(session.user_id),
        credential_id: Set(cred_id_b64.clone()),
        passkey_json: Set(passkey_json),
        created_at: Set(now),
        last_used_at: Set(None),
    };
    passkeys::Entity::insert(passkey_model)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let device_model = devices::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(session.user_id),
        device_type: Set("passkey".to_string()),
        label: Set(payload
            .device_label
            .unwrap_or_else(|| format!("Passkey-{passkey_id}"))),
        credential_id: Set(Some(cred_id_b64)),
        created_at: Set(now),
        last_used_at: Set(None),
    };
    devices::Entity::insert(device_model)
        .exec_without_returning(&state.db)
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
    if !user.is_active {
        return Err(AppError::auth("user disabled"));
    }
    if !user.is_active {
        return Err(AppError::auth("user disabled"));
    }

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
        devices::Entity::insert(device)
            .exec_without_returning(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    }

    let user = User::find_by_id(session.user_id)
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("user not found"))?;
    if !user.is_active {
        return Err(AppError::auth("user disabled"));
    }

    let (jar, user_id) = create_session_cookie(&state, jar, session.user_id).await?;

    Ok((jar, Json(PasskeyLoginFinishResponse { user_id })))
}

/// 密码登录（仅学生）。
pub async fn password_login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<PasswordLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::find()
        .filter(users::Column::Username.eq(payload.username))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("user not found"))?;
    if !user.is_active {
        return Err(AppError::auth("user disabled"));
    }
    if user.role != "student" || !user.allow_password_login {
        return Err(AppError::auth("password login not allowed"));
    }
    let hash = user
        .password_hash
        .as_ref()
        .ok_or_else(|| AppError::auth("password not set"))?;
    if !verify_password(&payload.password, hash)? {
        return Err(AppError::auth("invalid password"));
    }
    let (jar, user_id) = create_session_cookie(&state, jar, user.id).await?;
    Ok((jar, Json(serde_json::json!({"user_id": user_id}))))
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

    let (secret, url) = generate_totp("UCA Platform", &user.username)?;
    let encrypted = encrypt_secret(&secret, &state.config.auth_secret_key)?;

    let now = Utc::now();
    let enrollment_id = Uuid::new_v4();

    let totp_model = totp_secrets::ActiveModel {
        id: Set(enrollment_id),
        user_id: Set(user.id),
        secret_enc: Set(encrypted),
        enabled: Set(false),
        created_at: Set(now),
        verified_at: Set(None),
    };
    totp_secrets::Entity::insert(totp_model)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    if let Some(label) = payload.device_label {
        let device_model = devices::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user.id),
            device_type: Set("totp".to_string()),
            label: Set(label),
            credential_id: Set(None),
            created_at: Set(now),
            last_used_at: Set(None),
        };
        devices::Entity::insert(device_model)
            .exec_without_returning(&state.db)
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
    if !user.is_active {
        return Err(AppError::auth("user disabled"));
    }

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

/// 绑定学生邮箱（仅学生本人）。
pub async fn bind_email(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<EmailBindRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !is_valid_email(&payload.email) {
        return Err(AppError::validation("invalid email"));
    }
    let user = require_session(&state, &jar).await?;
    if user.role != "student" {
        return Err(AppError::auth("forbidden"));
    }
    let mut active: users::ActiveModel = user.into();
    active.email = Set(Some(payload.email));
    active.updated_at = Set(Utc::now());
    active
        .update(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// 学生修改密码。
pub async fn change_password(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<PasswordChangeRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session(&state, &jar).await?;
    if user.role != "student" || !user.allow_password_login {
        return Err(AppError::auth("forbidden"));
    }
    let hash = user
        .password_hash
        .as_ref()
        .ok_or_else(|| AppError::auth("password not set"))?;
    if !verify_password(&payload.current_password, hash)? {
        return Err(AppError::auth("invalid password"));
    }
    let policy = load_password_policy(&state).await?;
    validate_password_policy(&policy, &payload.new_password)?;
    let new_hash = hash_password(&payload.new_password)?;
    let mut active: users::ActiveModel = user.into();
    active.password_hash = Set(Some(new_hash));
    active.password_updated_at = Set(Some(Utc::now()));
    active.updated_at = Set(Utc::now());
    active
        .update(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// 学生发起密码重置邮件。
pub async fn password_reset_request(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if matches!(state.config.reset_delivery, crate::config::ResetDelivery::Code) {
        return Err(AppError::bad_request("reset delivery set to code"));
    }
    let user = User::find()
        .filter(users::Column::Username.eq(payload.username))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("user not found"))?;
    if user.role != "student" {
        return Err(AppError::auth("forbidden"));
    }
    let email = user.email.clone().ok_or_else(|| AppError::bad_request("email not bound"))?;
    let mail_config = state
        .config
        .mail
        .as_ref()
        .ok_or_else(|| AppError::config("mail config required"))?;
    let base_url = state
        .config
        .base_url
        .as_ref()
        .ok_or_else(|| AppError::config("BASE_URL is required"))?;

    let token = generate_token();
    let token_hash = hash_token(&token);
    let now = Utc::now();
    let expires_at = now + ChronoDuration::minutes(PASSWORD_RESET_TTL_MINUTES);

    let model = auth_resets::ActiveModel {
        id: Set(Uuid::new_v4()),
        token_hash: Set(token_hash),
        user_id: Set(user.id),
        purpose: Set("password".to_string()),
        expires_at: Set(expires_at),
        created_at: Set(now),
        used_at: Set(None),
    };
    auth_resets::Entity::insert(model)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let link = format!("{}/password-reset?token={}", base_url, token);
    let body = format!(
        "您好，\n\n请使用以下链接重置您的密码：\n{}\n\n该链接 {} 分钟后失效。",
        link, PASSWORD_RESET_TTL_MINUTES
    );
    send_mail(mail_config, &email, "密码重置", &body).await?;

    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// 完成学生密码重置。
pub async fn password_reset_confirm(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetConfirmRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let token_hash = hash_token(&payload.token);
    let record = AuthReset::find()
        .filter(auth_resets::Column::TokenHash.eq(token_hash))
        .filter(auth_resets::Column::Purpose.eq("password"))
        .filter(auth_resets::Column::UsedAt.is_null())
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::auth("invalid or expired token"))?;
    if record.expires_at < Utc::now() {
        return Err(AppError::auth("token expired"));
    }

    let user = User::find_by_id(record.user_id)
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("user not found"))?;
    if user.role != "student" {
        return Err(AppError::auth("forbidden"));
    }

    let policy = load_password_policy(&state).await?;
    validate_password_policy(&policy, &payload.new_password)?;
    let new_hash = hash_password(&payload.new_password)?;

    let mut user_active: users::ActiveModel = user.into();
    user_active.password_hash = Set(Some(new_hash));
    user_active.allow_password_login = Set(true);
    user_active.password_updated_at = Set(Some(Utc::now()));
    user_active.updated_at = Set(Utc::now());
    user_active
        .update(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut active: auth_resets::ActiveModel = record.into();
    active.used_at = Set(Some(Utc::now()));
    active
        .update(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// 获取邀请状态。
pub async fn invite_status(
    State(state): State<AppState>,
    Query(query): Query<InviteAcceptRequest>,
) -> Result<Json<InviteStatusResponse>, AppError> {
    let token_hash = hash_token(&query.token);
    let invite = Invite::find()
        .filter(invites::Column::TokenHash.eq(token_hash))
        .filter(invites::Column::UsedAt.is_null())
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    if let Some(record) = invite {
        if record.expires_at < Utc::now() {
            return Ok(Json(InviteStatusResponse {
                valid: false,
                email: None,
                username: None,
                display_name: None,
                role: None,
                expires_at: None,
            }));
        }
        return Ok(Json(InviteStatusResponse {
            valid: true,
            email: Some(record.email),
            username: Some(record.username),
            display_name: Some(record.display_name),
            role: Some(record.role),
            expires_at: Some(record.expires_at),
        }));
    }

    Ok(Json(InviteStatusResponse {
        valid: false,
        email: None,
        username: None,
        display_name: None,
        role: None,
        expires_at: None,
    }))
}

/// 接受邀请并创建用户。
pub async fn invite_accept(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<InviteAcceptRequest>,
) -> Result<(CookieJar, Json<InviteAcceptResponse>), AppError> {
    let token_hash = hash_token(&payload.token);
    let invite = Invite::find()
        .filter(invites::Column::TokenHash.eq(token_hash))
        .filter(invites::Column::UsedAt.is_null())
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::auth("invalid or expired invite"))?;
    let invite_username = invite.username.clone();
    let invite_role = invite.role.clone();
    if invite.expires_at < Utc::now() {
        return Err(AppError::auth("invite expired"));
    }

    let exists = User::find()
        .filter(users::Column::Username.eq(&invite.username))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    if exists.is_some() {
        return Err(AppError::bad_request("user already exists"));
    }

    let now = Utc::now();
    let user_id = Uuid::new_v4();
    let user = users::ActiveModel {
        id: Set(user_id),
        username: Set(invite.username.clone()),
        display_name: Set(invite.display_name.clone()),
        role: Set(invite.role.clone()),
        email: Set(Some(invite.email.clone())),
        password_hash: Set(None),
        allow_password_login: Set(false),
        password_updated_at: Set(None),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    };
    users::Entity::insert(user)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut invite_active: invites::ActiveModel = invite.into();
    invite_active.used_at = Set(Some(Utc::now()));
    invite_active
        .update(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let (jar, _) = create_session_cookie(&state, jar, user_id).await?;
    Ok((
        jar,
        Json(InviteAcceptResponse {
            user_id,
            username: invite_username,
            role: invite_role,
        }),
    ))
}

/// 获取重置令牌状态。
pub async fn reset_status(
    State(state): State<AppState>,
    Query(query): Query<ResetConsumeRequest>,
) -> Result<Json<ResetStatusResponse>, AppError> {
    let token_hash = hash_token(&query.token);
    let record = AuthReset::find()
        .filter(auth_resets::Column::TokenHash.eq(token_hash))
        .filter(auth_resets::Column::UsedAt.is_null())
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    if let Some(reset) = record {
        if reset.expires_at < Utc::now() {
            return Ok(Json(ResetStatusResponse { valid: false, purpose: None }));
        }
        return Ok(Json(ResetStatusResponse {
            valid: true,
            purpose: Some(reset.purpose),
        }));
    }
    Ok(Json(ResetStatusResponse { valid: false, purpose: None }))
}

/// 消费重置令牌并清理认证数据（TOTP/Passkey）。
pub async fn reset_consume(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<ResetConsumeRequest>,
) -> Result<(CookieJar, Json<ResetConsumeResponse>), AppError> {
    let token_hash = hash_token(&payload.token);
    let record = AuthReset::find()
        .filter(auth_resets::Column::TokenHash.eq(token_hash))
        .filter(auth_resets::Column::UsedAt.is_null())
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::auth("invalid or expired token"))?;
    let user_id = record.user_id;
    if record.expires_at < Utc::now() {
        return Err(AppError::auth("token expired"));
    }

    let purpose = record.purpose.clone();
    if purpose == "totp" {
        totp_secrets::Entity::delete_many()
            .filter(totp_secrets::Column::UserId.eq(record.user_id))
            .exec(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
        devices::Entity::delete_many()
            .filter(devices::Column::UserId.eq(record.user_id))
            .filter(devices::Column::DeviceType.eq("totp"))
            .exec(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    } else if purpose == "passkey" {
        passkeys::Entity::delete_many()
            .filter(passkeys::Column::UserId.eq(record.user_id))
            .exec(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
        devices::Entity::delete_many()
            .filter(devices::Column::UserId.eq(record.user_id))
            .filter(devices::Column::DeviceType.eq("passkey"))
            .exec(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    } else {
        return Err(AppError::bad_request("invalid reset purpose"));
    }

    sessions::Entity::delete_many()
        .filter(sessions::Column::UserId.eq(record.user_id))
        .exec(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut active: auth_resets::ActiveModel = record.into();
    active.used_at = Set(Some(Utc::now()));
    active
        .update(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let (jar, _) = create_session_cookie(&state, jar, user_id).await?;
    Ok((
        jar,
        Json(ResetConsumeResponse {
            user_id,
            purpose,
        }),
    ))
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
        let recovery_model = recovery_codes::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user.id),
            code_hash: Set(code.hash.clone()),
            used_at: Set(None),
            created_at: Set(now),
        };
        recovery_codes::Entity::insert(recovery_model)
            .exec_without_returning(&state.db)
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

fn validate_password_policy(
    policy: &crate::config::PasswordPolicy,
    password: &str,
) -> Result<(), AppError> {
    if password.len() < policy.min_length {
        return Err(AppError::validation("password too short"));
    }
    if policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
        return Err(AppError::validation("password requires uppercase"));
    }
    if policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
        return Err(AppError::validation("password requires lowercase"));
    }
    if policy.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(AppError::validation("password requires digit"));
    }
    if policy.require_symbol && !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(AppError::validation("password requires symbol"));
    }
    Ok(())
}

fn is_valid_email(value: &str) -> bool {
    let mut parts = value.split('@');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(local), Some(domain), None) => !local.is_empty() && domain.contains('.'),
        _ => false,
    }
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

    let session_model = sessions::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        token_hash: Set(token_hash),
        expires_at: Set(expires_db),
        created_at: Set(now_db),
        last_seen_at: Set(Some(now_db)),
    };
    sessions::Entity::insert(session_model)
        .exec_without_returning(&state.db)
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
