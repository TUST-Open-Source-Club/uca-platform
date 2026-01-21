//! 会话与角色访问控制辅助。

use axum_extra::extract::cookie::CookieJar;
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    auth::hash_session_token,
    entities::{sessions, users, Session, User},
    error::AppError,
    state::AppState,
};

/// 要求有效会话并返回用户模型。
pub async fn require_session_user(
    state: &AppState,
    jar: &CookieJar,
) -> Result<users::Model, AppError> {
    let token = jar
        .get(&state.config.session_cookie_name)
        .ok_or_else(|| AppError::auth("missing session"))?
        .value()
        .to_string();
    let token_hash = hash_session_token(&token);

    let session = Session::find()
        .filter(sessions::Column::TokenHash.eq(token_hash))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::auth("invalid session"))?;

    if session.expires_at < Utc::now() {
        return Err(AppError::auth("session expired"));
    }

    User::find_by_id(session.user_id)
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::auth("user not found"))
}

/// 确认用户具备指定角色。
pub fn require_role(user: &users::Model, role: &str) -> Result<(), AppError> {
    if user.role == role {
        Ok(())
    } else {
        Err(AppError::auth("forbidden"))
    }
}
