//! 志愿服务与竞赛记录接口。

use axum::{extract::State, Json, extract::Path};
use axum_extra::extract::cookie::CookieJar;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
    access::{require_role, require_session_user},
    entities::{
        competition_library, contest_records, students, volunteer_records, CompetitionLibrary,
        ContestRecord, Student, VolunteerRecord,
    },
    error::AppError,
    state::AppState,
};

const STATUS_SUBMITTED: &str = "submitted";
const STATUS_FIRST_REVIEWED: &str = "first_reviewed";
const STATUS_FINAL_REVIEWED: &str = "final_reviewed";
const STATUS_REJECTED: &str = "rejected";

const REVIEW_STAGE_FIRST: &str = "first";
const REVIEW_STAGE_FINAL: &str = "final";

/// 志愿服务提交请求。
#[derive(Debug, Deserialize, Validate)]
pub struct CreateVolunteerRequest {
    /// 标题。
    #[validate(length(min = 1, max = 120))]
    pub title: String,
    /// 描述。
    #[validate(length(min = 1, max = 2000))]
    pub description: String,
    /// 自评学时。
    pub self_hours: i32,
}

/// 竞赛获奖提交请求。
#[derive(Debug, Deserialize, Validate)]
pub struct CreateContestRequest {
    /// 竞赛名称。
    #[validate(length(min = 1, max = 200))]
    pub contest_name: String,
    /// 获奖等级。
    #[validate(length(min = 1, max = 120))]
    pub award_level: String,
    /// 自评学时。
    pub self_hours: i32,
}

/// 志愿服务记录响应。
#[derive(Debug, Serialize)]
pub struct VolunteerRecordResponse {
    /// 记录 ID。
    pub id: Uuid,
    /// 学生 ID。
    pub student_id: Uuid,
    /// 标题。
    pub title: String,
    /// 描述。
    pub description: String,
    /// 自评学时。
    pub self_hours: i32,
    /// 初审学时。
    pub first_review_hours: Option<i32>,
    /// 复审学时。
    pub final_review_hours: Option<i32>,
    /// 状态。
    pub status: String,
    /// 不通过原因。
    pub rejection_reason: Option<String>,
}

/// 竞赛记录响应。
#[derive(Debug, Serialize)]
pub struct ContestRecordResponse {
    /// 记录 ID。
    pub id: Uuid,
    /// 学生 ID。
    pub student_id: Uuid,
    /// 竞赛名称。
    pub contest_name: String,
    /// 获奖等级。
    pub award_level: String,
    /// 自评学时。
    pub self_hours: i32,
    /// 初审学时。
    pub first_review_hours: Option<i32>,
    /// 复审学时。
    pub final_review_hours: Option<i32>,
    /// 状态。
    pub status: String,
    /// 不通过原因。
    pub rejection_reason: Option<String>,
    /// 竞赛名称匹配标识。
    pub match_status: String,
}

/// 志愿服务查询条件。
#[derive(Debug, Deserialize)]
pub struct VolunteerQuery {
    /// 状态筛选。
    pub status: Option<String>,
}

/// 竞赛查询条件。
#[derive(Debug, Deserialize)]
pub struct ContestQuery {
    /// 状态筛选。
    pub status: Option<String>,
}

/// 审核请求。
#[derive(Debug, Deserialize, Validate)]
pub struct ReviewRequest {
    /// 审核阶段：first/final。
    #[validate(length(min = 1, max = 16))]
    pub stage: String,
    /// 审核学时。
    pub hours: i32,
    /// 状态：approved/rejected。
    #[validate(length(min = 1, max = 16))]
    pub status: String,
    /// 不通过原因。
    pub rejection_reason: Option<String>,
}

/// 提交志愿服务记录（学生）。
pub async fn create_volunteer_record(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<CreateVolunteerRequest>,
) -> Result<Json<VolunteerRecordResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "student")?;
    payload
        .validate()
        .map_err(|_| AppError::validation("invalid volunteer payload"))?;

    let student = Student::find()
        .filter(students::Column::StudentNo.eq(&user.username))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("student not found"))?;

    let now = Utc::now();
    let model = volunteer_records::ActiveModel {
        id: Set(Uuid::new_v4()),
        student_id: Set(student.id),
        title: Set(payload.title),
        description: Set(payload.description),
        self_hours: Set(payload.self_hours),
        first_review_hours: Set(None),
        final_review_hours: Set(None),
        status: Set(STATUS_SUBMITTED.to_string()),
        rejection_reason: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&state.db)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(model_to_volunteer_response(model)))
}

/// 提交竞赛获奖记录（学生）。
pub async fn create_contest_record(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<CreateContestRequest>,
) -> Result<Json<ContestRecordResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "student")?;
    payload
        .validate()
        .map_err(|_| AppError::validation("invalid contest payload"))?;

    let student = Student::find()
        .filter(students::Column::StudentNo.eq(&user.username))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("student not found"))?;

    let now = Utc::now();
    let model = contest_records::ActiveModel {
        id: Set(Uuid::new_v4()),
        student_id: Set(student.id),
        contest_name: Set(payload.contest_name),
        award_level: Set(payload.award_level),
        self_hours: Set(payload.self_hours),
        first_review_hours: Set(None),
        final_review_hours: Set(None),
        status: Set(STATUS_SUBMITTED.to_string()),
        rejection_reason: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&state.db)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    let match_status = contest_match_status(&state, &model.contest_name).await?;
    Ok(Json(model_to_contest_response(model, &match_status)))
}

/// 查询志愿服务记录（学生或审核角色）。
pub async fn list_volunteer_records(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(query): Json<VolunteerQuery>,
) -> Result<Json<Vec<VolunteerRecordResponse>>, AppError> {
    let user = require_session_user(&state, &jar).await?;

    let mut finder = VolunteerRecord::find();
    if user.role == "student" {
        let student = Student::find()
            .filter(students::Column::StudentNo.eq(&user.username))
            .one(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?
            .ok_or_else(|| AppError::not_found("student not found"))?;
        finder = finder.filter(volunteer_records::Column::StudentId.eq(student.id));
    } else if user.role != "admin" && user.role != "teacher" && user.role != "reviewer" {
        return Err(AppError::auth("forbidden"));
    }

    if let Some(status) = query.status {
        finder = finder.filter(volunteer_records::Column::Status.eq(status));
    }

    let records = finder
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(records.into_iter().map(model_to_volunteer_response).collect()))
}

/// 查询竞赛记录（学生或审核角色）。
pub async fn list_contest_records(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(query): Json<ContestQuery>,
) -> Result<Json<Vec<ContestRecordResponse>>, AppError> {
    let user = require_session_user(&state, &jar).await?;

    let mut finder = ContestRecord::find();
    if user.role == "student" {
        let student = Student::find()
            .filter(students::Column::StudentNo.eq(&user.username))
            .one(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?
            .ok_or_else(|| AppError::not_found("student not found"))?;
        finder = finder.filter(contest_records::Column::StudentId.eq(student.id));
    } else if user.role != "admin" && user.role != "teacher" && user.role != "reviewer" {
        return Err(AppError::auth("forbidden"));
    }

    if let Some(status) = query.status {
        finder = finder.filter(contest_records::Column::Status.eq(status));
    }

    let records = finder
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut responses = Vec::with_capacity(records.len());
    for record in records {
        let match_status = contest_match_status(&state, &record.contest_name).await?;
        responses.push(model_to_contest_response(record, &match_status));
    }

    Ok(Json(responses))
}

/// 审核志愿服务记录（审核人员/教师）。
pub async fn review_volunteer_record(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(record_id): Path<Uuid>,
    Json(payload): Json<ReviewRequest>,
) -> Result<Json<VolunteerRecordResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    ensure_review_permission(&user, &payload.stage)?;
    payload
        .validate()
        .map_err(|_| AppError::validation("invalid review payload"))?;

    let record = VolunteerRecord::find_by_id(record_id)
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("record not found"))?;

    let mut active: volunteer_records::ActiveModel = record.into();
    apply_review_update(&payload, &mut active.status, &mut active.rejection_reason)?;
    if payload.stage == REVIEW_STAGE_FIRST {
        active.first_review_hours = Set(Some(payload.hours));
    } else {
        active.final_review_hours = Set(Some(payload.hours));
    }
    active.updated_at = Set(Utc::now());

    let model = active
        .update(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(model_to_volunteer_response(model)))
}

/// 审核竞赛记录（审核人员/教师）。
pub async fn review_contest_record(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(record_id): Path<Uuid>,
    Json(payload): Json<ReviewRequest>,
) -> Result<Json<ContestRecordResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    ensure_review_permission(&user, &payload.stage)?;
    payload
        .validate()
        .map_err(|_| AppError::validation("invalid review payload"))?;

    let record = ContestRecord::find_by_id(record_id)
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("record not found"))?;

    let mut active: contest_records::ActiveModel = record.into();
    apply_review_update(&payload, &mut active.status, &mut active.rejection_reason)?;
    if payload.stage == REVIEW_STAGE_FIRST {
        active.first_review_hours = Set(Some(payload.hours));
    } else {
        active.final_review_hours = Set(Some(payload.hours));
    }
    active.updated_at = Set(Utc::now());

    let model = active
        .update(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let match_status = contest_match_status(&state, &model.contest_name).await?;
    Ok(Json(model_to_contest_response(model, &match_status)))
}

fn model_to_volunteer_response(model: volunteer_records::Model) -> VolunteerRecordResponse {
    VolunteerRecordResponse {
        id: model.id,
        student_id: model.student_id,
        title: model.title,
        description: model.description,
        self_hours: model.self_hours,
        first_review_hours: model.first_review_hours,
        final_review_hours: model.final_review_hours,
        status: model.status,
        rejection_reason: model.rejection_reason,
    }
}

fn model_to_contest_response(
    model: contest_records::Model,
    match_status: &str,
) -> ContestRecordResponse {
    ContestRecordResponse {
        id: model.id,
        student_id: model.student_id,
        contest_name: model.contest_name,
        award_level: model.award_level,
        self_hours: model.self_hours,
        first_review_hours: model.first_review_hours,
        final_review_hours: model.final_review_hours,
        status: model.status,
        rejection_reason: model.rejection_reason,
        match_status: match_status.to_string(),
    }
}

async fn contest_match_status(state: &AppState, contest_name: &str) -> Result<String, AppError> {
    let match_exists = CompetitionLibrary::find()
        .filter(competition_library::Column::Name.eq(contest_name))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .is_some();

    if match_exists {
        Ok("matched".to_string())
    } else {
        Ok("unmatched".to_string())
    }
}

fn ensure_review_permission(user: &crate::entities::users::Model, stage: &str) -> Result<(), AppError> {
    if stage == REVIEW_STAGE_FIRST && (user.role == "reviewer" || user.role == "admin") {
        return Ok(());
    }
    if stage == REVIEW_STAGE_FINAL && (user.role == "teacher" || user.role == "admin") {
        return Ok(());
    }
    Err(AppError::auth("forbidden"))
}

fn apply_review_update(
    payload: &ReviewRequest,
    status: &mut sea_orm::ActiveValue<String>,
    rejection_reason: &mut sea_orm::ActiveValue<Option<String>>,
) -> Result<(), AppError> {
    if payload.status == "rejected" {
        *status = Set(STATUS_REJECTED.to_string());
        *rejection_reason = Set(payload.rejection_reason.clone());
        return Ok(());
    }

    if payload.stage == REVIEW_STAGE_FIRST {
        *status = Set(STATUS_FIRST_REVIEWED.to_string());
    } else if payload.stage == REVIEW_STAGE_FINAL {
        *status = Set(STATUS_FINAL_REVIEWED.to_string());
    } else {
        return Err(AppError::bad_request("invalid review stage"));
    }

    *rejection_reason = Set(None);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_review_update_rejects() {
        let payload = ReviewRequest {
            stage: REVIEW_STAGE_FIRST.to_string(),
            hours: 2,
            status: "rejected".to_string(),
            rejection_reason: Some("no proof".to_string()),
        };
        let mut status = sea_orm::ActiveValue::set("".to_string());
        let mut reason = sea_orm::ActiveValue::set(None);
        apply_review_update(&payload, &mut status, &mut reason).expect("apply");
        assert_eq!(status.unwrap(), STATUS_REJECTED.to_string());
        assert_eq!(reason.unwrap(), Some("no proof".to_string()));
    }

    #[test]
    fn apply_review_update_first_pass() {
        let payload = ReviewRequest {
            stage: REVIEW_STAGE_FIRST.to_string(),
            hours: 2,
            status: "approved".to_string(),
            rejection_reason: None,
        };
        let mut status = sea_orm::ActiveValue::set("".to_string());
        let mut reason = sea_orm::ActiveValue::set(None);
        apply_review_update(&payload, &mut status, &mut reason).expect("apply");
        assert_eq!(status.unwrap(), STATUS_FIRST_REVIEWED.to_string());
        assert_eq!(reason.unwrap(), None);
    }
}
