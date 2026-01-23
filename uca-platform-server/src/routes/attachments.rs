//! 附件与签名上传接口。

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderValue, StatusCode},
    response::Response,
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Serialize;
use std::path::{Path as StdPath, PathBuf};
use tokio::fs;
use uuid::Uuid;

use crate::{
    access::require_session_user,
    entities::{attachments, review_signatures, students, Attachment, ContestRecord, Student},
    error::AppError,
    state::AppState,
};

const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

/// 附件上传响应。
#[derive(Debug, Serialize)]
pub struct AttachmentResponse {
    /// 附件 ID。
    pub id: Uuid,
    /// 存储文件名。
    pub stored_name: String,
}

/// 签名上传响应。
#[derive(Debug, Serialize)]
pub struct SignatureResponse {
    /// 签名 ID。
    pub id: Uuid,
    /// 存储路径。
    pub signature_path: String,
}

/// 上传竞赛附件（学生本人）。
pub async fn upload_contest_attachment(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(record_id): Path<Uuid>,
    multipart: Multipart,
) -> Result<Json<AttachmentResponse>, AppError> {
    upload_record_attachment(&state, &jar, "contest", record_id, multipart).await
}

/// 上传审核签名（初审/复审）。
pub async fn upload_review_signature(
    State(state): State<AppState>,
    jar: CookieJar,
    Path((record_type, record_id, stage)): Path<(String, Uuid, String)>,
    multipart: Multipart,
) -> Result<Json<SignatureResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    ensure_review_permission(&user.role, &stage)?;

    let student = match record_type.as_str() {
        "contest" => {
            let record = ContestRecord::find_by_id(record_id)
                .one(&state.db)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?
                .ok_or_else(|| AppError::not_found("record not found"))?;
            if record.is_deleted {
                return Err(AppError::not_found("record not found"));
            }
            Student::find_by_id(record.student_id)
                .one(&state.db)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?
                .ok_or_else(|| AppError::not_found("student not found"))
                .and_then(|student| {
                    if student.is_deleted {
                        Err(AppError::not_found("student not found"))
                    } else {
                        Ok(student)
                    }
                })?
        }
        _ => return Err(AppError::bad_request("invalid record type")),
    };

    let (bytes, original_name, _mime_type) = read_multipart_file(multipart).await?;
    let stored_name = build_stored_name(
        &student.student_no,
        &student.name,
        "signature",
        &original_name,
    );
    let dir = build_upload_dir(&state.config.upload_dir, "signatures", &record_type, Some(&stage));
    let path = save_bytes(&dir, &stored_name, &bytes).await?;

    let id = Uuid::new_v4();
    let model = review_signatures::ActiveModel {
        id: Set(id),
        record_type: Set(record_type),
        record_id: Set(record_id),
        reviewer_user_id: Set(user.id),
        stage: Set(stage),
        signature_path: Set(path.to_string_lossy().to_string()),
        created_at: Set(Utc::now()),
    };
    review_signatures::Entity::insert(model)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(SignatureResponse {
        id,
        signature_path: path.to_string_lossy().to_string(),
    }))
}

/// 下载附件（审核人员/管理员/学生本人）。
pub async fn download_attachment(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(attachment_id): Path<Uuid>,
) -> Result<Response, AppError> {
    let user = require_session_user(&state, &jar).await?;
    let attachment = Attachment::find_by_id(attachment_id)
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("attachment not found"))?;

    if user.role == "student" {
        let student = Student::find()
            .filter(students::Column::StudentNo.eq(&user.username))
            .filter(students::Column::IsDeleted.eq(false))
            .one(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?
            .ok_or_else(|| AppError::not_found("student not found"))?;
        if attachment.student_id != student.id {
            return Err(AppError::auth("forbidden"));
        }
    } else if user.role != "admin" && user.role != "reviewer" && user.role != "teacher" {
        return Err(AppError::auth("forbidden"));
    }

    let bytes = fs::read(&attachment.stored_name)
        .await
        .map_err(|_| AppError::not_found("file not found"))?;

    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = StatusCode::OK;
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&attachment.mime_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    let disposition = format!(
        "inline; filename=\"{}\"",
        attachment.original_name.replace('"', "_")
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&disposition)
            .unwrap_or_else(|_| HeaderValue::from_static("inline")),
    );
    Ok(response)
}

async fn upload_record_attachment(
    state: &AppState,
    jar: &CookieJar,
    record_type: &str,
    record_id: Uuid,
    multipart: Multipart,
) -> Result<Json<AttachmentResponse>, AppError> {
    let user = require_session_user(state, jar).await?;
    if user.role != "student" {
        return Err(AppError::auth("forbidden"));
    }

    let student = Student::find()
        .filter(students::Column::StudentNo.eq(&user.username))
        .filter(students::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("student not found"))?;

    ensure_record_ownership(state, record_type, record_id, student.id).await?;

    let (bytes, original_name, mime_type) = read_multipart_file(multipart).await?;
    if !is_supported_attachment(&mime_type) {
        return Err(AppError::bad_request("unsupported file type"));
    }
    let stored_name = build_stored_name(
        &student.student_no,
        &student.name,
        record_type,
        &original_name,
    );
    let dir = build_upload_dir(&state.config.upload_dir, "attachments", record_type, None);
    let path = save_bytes(&dir, &stored_name, &bytes).await?;

    let id = Uuid::new_v4();
    let model = attachments::ActiveModel {
        id: Set(id),
        student_id: Set(student.id),
        record_type: Set(record_type.to_string()),
        record_id: Set(record_id),
        original_name: Set(original_name),
        stored_name: Set(path.to_string_lossy().to_string()),
        mime_type: Set(mime_type),
        created_at: Set(Utc::now()),
    };
    attachments::Entity::insert(model)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(AttachmentResponse {
        id,
        stored_name: path.to_string_lossy().to_string(),
    }))
}

async fn ensure_record_ownership(
    state: &AppState,
    record_type: &str,
    record_id: Uuid,
    student_id: Uuid,
) -> Result<(), AppError> {
    match record_type {
        "contest" => {
            let record = ContestRecord::find_by_id(record_id)
                .one(&state.db)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?
                .ok_or_else(|| AppError::not_found("record not found"))?;
            if record.is_deleted {
                return Err(AppError::not_found("record not found"));
            }
            if record.student_id != student_id {
                return Err(AppError::auth("forbidden"));
            }
        }
        _ => return Err(AppError::bad_request("invalid record type")),
    }
    Ok(())
}

async fn read_multipart_file(mut multipart: Multipart) -> Result<(Vec<u8>, String, String), AppError> {
    let mut file_bytes = None;
    let mut filename = None;
    let mut mime_type = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::bad_request("invalid multipart"))?
    {
        if field.name() != Some("file") {
            continue;
        }
        let name = field
            .file_name()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "upload.bin".to_string());
        let content_type = field
            .content_type()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());
        let bytes = field
            .bytes()
            .await
            .map_err(|_| AppError::bad_request("failed to read file"))?;
        if bytes.len() > MAX_UPLOAD_BYTES {
            return Err(AppError::bad_request("file too large"));
        }
        file_bytes = Some(bytes.to_vec());
        filename = Some(name);
        mime_type = Some(content_type);
        break;
    }

    let bytes = file_bytes.ok_or_else(|| AppError::bad_request("file field required"))?;
    let filename = filename.ok_or_else(|| AppError::bad_request("file name required"))?;
    let mime_type = mime_type.ok_or_else(|| AppError::bad_request("mime required"))?;

    Ok((bytes, filename, mime_type))
}

fn is_supported_attachment(mime_type: &str) -> bool {
    let lower = mime_type.to_ascii_lowercase();
    lower == "application/pdf" || lower.starts_with("image/")
}

fn build_stored_name(student_no: &str, name: &str, file_type: &str, original: &str) -> String {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let ext = StdPath::new(original)
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or("");
    let ext = if ext.is_empty() { "bin" } else { ext };
    let ext = format!(".{ext}");
    let safe_no = sanitize_component(student_no);
    let safe_name = sanitize_component(name);
    let safe_type = sanitize_component(file_type);
    format!("{safe_no}_{safe_name}_{safe_type}_{timestamp}{ext}")
}

fn sanitize_component(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch == '/' || ch == '\\' || ch == '\0' || ch == ':' {
                '_'
            } else {
                ch
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn build_upload_dir(base: &PathBuf, category: &str, record_type: &str, stage: Option<&str>) -> PathBuf {
    let mut dir = base.join(category).join(record_type);
    if let Some(stage) = stage {
        dir = dir.join(stage);
    }
    dir
}

async fn save_bytes(dir: &StdPath, stored_name: &str, bytes: &[u8]) -> Result<PathBuf, AppError> {
    fs::create_dir_all(dir)
        .await
        .map_err(|err| AppError::internal(&format!("create dir failed: {err}")))?;

    let path = dir.join(stored_name);
    fs::write(&path, bytes)
        .await
        .map_err(|err| AppError::internal(&format!("write file failed: {err}")))?;

    Ok(path)
}

fn ensure_review_permission(role: &str, stage: &str) -> Result<(), AppError> {
    if stage == "first" && (role == "reviewer" || role == "admin") {
        return Ok(());
    }
    if stage == "final" && (role == "teacher" || role == "admin") {
        return Ok(());
    }
    Err(AppError::auth("forbidden"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_component_replaces_separators() {
        let value = sanitize_component("a/b\\c");
        assert_eq!(value, "a_b_c");
    }

    #[test]
    fn build_stored_name_sanitizes_and_preserves_extension() {
        let name = build_stored_name("2023/01", "张/三", "contest:", "proof.pdf");
        assert!(name.contains("2023_01"));
        assert!(name.contains("张_三"));
        assert!(name.contains("contest_"));
        assert!(name.ends_with(".pdf"));
    }

    #[test]
    fn build_stored_name_falls_back_to_bin() {
        let name = build_stored_name("2023", "张三", "contest", "proof");
        assert!(name.ends_with(".bin"));
    }

    #[test]
    fn build_upload_dir_appends_stage() {
        let base = PathBuf::from("data/uploads");
        let dir = build_upload_dir(&base, "signatures", "contest", Some("first"));
        assert!(dir.ends_with("data/uploads/signatures/contest/first"));
    }

    #[test]
    fn ensure_review_permission_allows_expected_roles() {
        assert!(ensure_review_permission("reviewer", "first").is_ok());
        assert!(ensure_review_permission("teacher", "final").is_ok());
        assert!(ensure_review_permission("student", "first").is_err());
    }
}
