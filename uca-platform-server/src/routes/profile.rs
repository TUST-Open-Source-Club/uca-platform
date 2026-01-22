//! 个人中心（签名图片管理）。

use axum::{extract::{Multipart, State}, Json};
use axum_extra::extract::cookie::CookieJar;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::Serialize;
use tokio::fs;

use crate::{
    access::require_session_user,
    entities::{user_signatures, UserSignature},
    error::AppError,
    state::AppState,
};

const MAX_UPLOAD_BYTES: usize = 5 * 1024 * 1024;

/// 当前用户签名信息。
#[derive(Debug, Serialize)]
pub struct SignatureProfile {
    /// 是否已上传签名。
    pub uploaded: bool,
    /// 签名文件路径（仅用于显示状态）。
    pub signature_path: Option<String>,
}

/// 获取当前用户签名。
pub async fn get_signature(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<SignatureProfile>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    let record = UserSignature::find_by_id(user.id)
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(Json(SignatureProfile {
        uploaded: record.is_some(),
        signature_path: record.map(|item| item.signature_path),
    }))
}

/// 上传当前用户签名图片（审核人员/管理员/教师）。
pub async fn upload_signature(
    State(state): State<AppState>,
    jar: CookieJar,
    multipart: Multipart,
) -> Result<Json<SignatureProfile>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    if user.role != "admin" && user.role != "reviewer" && user.role != "teacher" {
        return Err(AppError::auth("forbidden"));
    }

    let (bytes, original_name) = read_signature_file(multipart).await?;
    let filename = build_signature_filename(&original_name);
    let dir = state
        .config
        .upload_dir
        .join("signatures")
        .join("users")
        .join(user.id.to_string());
    fs::create_dir_all(&dir)
        .await
        .map_err(|err| AppError::internal(&format!("failed to create dir: {err}")))?;
    let path = dir.join(filename);
    fs::write(&path, bytes)
        .await
        .map_err(|err| AppError::internal(&format!("failed to write file: {err}")))?;

    let now = Utc::now();
    if let Some(existing) = UserSignature::find_by_id(user.id)
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
    {
        let mut active: user_signatures::ActiveModel = existing.into();
        active.signature_path = Set(path.to_string_lossy().to_string());
        active.updated_at = Set(now);
        active
            .update(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    } else {
        let model = user_signatures::ActiveModel {
            user_id: Set(user.id),
            signature_path: Set(path.to_string_lossy().to_string()),
            created_at: Set(now),
            updated_at: Set(now),
        };
        user_signatures::Entity::insert(model)
            .exec_without_returning(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    }

    Ok(Json(SignatureProfile {
        uploaded: true,
        signature_path: Some(path.to_string_lossy().to_string()),
    }))
}

async fn read_signature_file(mut multipart: Multipart) -> Result<(Vec<u8>, String), AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::bad_request("invalid multipart"))?
    {
        if field.name() != Some("file") {
            continue;
        }
        let filename = field
            .file_name()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "signature.png".to_string());
        let bytes = field
            .bytes()
            .await
            .map_err(|_| AppError::bad_request("failed to read file"))?;
        if bytes.len() > MAX_UPLOAD_BYTES {
            return Err(AppError::bad_request("signature file too large"));
        }
        return Ok((bytes.to_vec(), filename));
    }
    Err(AppError::bad_request("file field required"))
}

fn build_signature_filename(original: &str) -> String {
    let ext = std::path::Path::new(original)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("png");
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    format!("signature_{timestamp}.{ext}")
}
