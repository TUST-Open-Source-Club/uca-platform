//! 竞赛记录接口。

use axum::{extract::State, Json, extract::Path};
use axum_extra::extract::cookie::CookieJar;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait,
    Set,
};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
    access::{require_role, require_session_user},
    entities::{
        competition_library, contest_records, form_field_values, form_fields, students,
        CompetitionLibrary, ContestRecord, FormField, FormFieldValue, Student,
    },
    error::AppError,
    labor_hours::{compute_recommended_hours, load_labor_hour_rules},
    state::AppState,
};

const STATUS_SUBMITTED: &str = "submitted";
const STATUS_FIRST_REVIEWED: &str = "first_reviewed";
const STATUS_FINAL_REVIEWED: &str = "final_reviewed";
const STATUS_REJECTED: &str = "rejected";

const REVIEW_STAGE_FIRST: &str = "first";
const REVIEW_STAGE_FINAL: &str = "final";

/// 竞赛获奖提交请求。
#[derive(Debug, Deserialize, Validate)]
pub struct CreateContestRequest {
    /// 竞赛名称。
    #[validate(length(min = 1, max = 200))]
    pub contest_name: String,
    /// 竞赛级别（国家级/省级/校级）。
    pub contest_level: Option<String>,
    /// 竞赛角色（负责人/成员）。
    pub contest_role: Option<String>,
    /// 竞赛年份。
    pub contest_year: Option<i32>,
    /// 竞赛类型（A/B）。
    pub contest_category: Option<String>,
    /// 获奖等级。
    #[validate(length(min = 1, max = 120))]
    pub award_level: String,
    /// 获奖时间（ISO 8601 日期或时间）。
    pub award_date: Option<String>,
    /// 自评学时。
    pub self_hours: i32,
    /// 自定义字段。
    pub custom_fields: Option<HashMap<String, String>>,
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
    /// 竞赛年份。
    pub contest_year: Option<i32>,
    /// 竞赛类型。
    pub contest_category: Option<String>,
    /// 竞赛级别。
    pub contest_level: Option<String>,
    /// 竞赛角色。
    pub contest_role: Option<String>,
    /// 获奖等级。
    pub award_level: String,
    /// 获奖时间。
    pub award_date: Option<String>,
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
    /// 推荐学时。
    pub recommended_hours: i32,
    /// 自定义字段。
    pub custom_fields: Vec<CustomFieldValueResponse>,
}

/// 自定义字段响应。
#[derive(Clone, Debug, Serialize)]
pub struct CustomFieldValueResponse {
    /// 字段 key。
    pub field_key: String,
    /// 字段标签。
    pub label: String,
    /// 字段值。
    pub value: String,
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
    if payload
        .contest_level
        .as_deref()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        return Err(AppError::validation("contest_level required"));
    }
    if payload
        .contest_role
        .as_deref()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        return Err(AppError::validation("contest_role required"));
    }

    let student = Student::find()
        .filter(students::Column::StudentNo.eq(&user.username))
        .filter(students::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("student not found"))?;

    let custom_fields = payload.custom_fields.unwrap_or_default();
    let form_fields = load_form_fields(&state, "contest").await?;
    validate_custom_fields(&form_fields, &custom_fields)?;

    let now = Utc::now();
    let id = Uuid::new_v4();
    let award_date = parse_award_date(payload.award_date.as_deref())?;
    let model = contest_records::ActiveModel {
        id: Set(id),
        student_id: Set(student.id),
        contest_year: Set(payload.contest_year),
        contest_category: Set(payload.contest_category.as_ref().map(|value| value.to_uppercase())),
        contest_name: Set(payload.contest_name.clone()),
        contest_level: Set(payload.contest_level.clone()),
        contest_role: Set(payload.contest_role.clone()),
        award_level: Set(payload.award_level.clone()),
        award_date: Set(award_date),
        self_hours: Set(payload.self_hours),
        first_review_hours: Set(None),
        final_review_hours: Set(None),
        status: Set(STATUS_SUBMITTED.to_string()),
        rejection_reason: Set(None),
        is_deleted: Set(false),
        created_at: Set(now),
        updated_at: Set(now),
    };
    contest_records::Entity::insert(model)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let match_status = contest_match_status(&state, &payload.contest_name).await?;
    let rule_config = load_labor_hour_rules(&state).await?;
    let recommended_hours = compute_recommended_hours(
        rule_config,
        payload.contest_category.as_deref(),
        payload.contest_level.as_deref(),
        payload.contest_role.as_deref(),
    );
    let model_id = id;
    insert_custom_fields(&state, "contest", model_id, &form_fields, &custom_fields).await?;
    let custom_values = fetch_custom_fields(&state, "contest", &[model_id], &form_fields).await?;
    let model = contest_records::Model {
        id,
        student_id: student.id,
        contest_year: payload.contest_year,
        contest_category: payload.contest_category.map(|value| value.to_uppercase()),
        contest_name: payload.contest_name,
        contest_level: payload.contest_level,
        contest_role: payload.contest_role,
        award_level: payload.award_level,
        award_date,
        self_hours: payload.self_hours,
        first_review_hours: None,
        final_review_hours: None,
        status: STATUS_SUBMITTED.to_string(),
        rejection_reason: None,
        is_deleted: false,
        created_at: now,
        updated_at: now,
    };
    Ok(Json(model_to_contest_response(
        model,
        &match_status,
        recommended_hours,
        custom_values.get(&model_id).cloned().unwrap_or_default(),
    )))
}

/// 查询竞赛记录（学生或审核角色）。
pub async fn list_contest_records(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(query): Json<ContestQuery>,
) -> Result<Json<Vec<ContestRecordResponse>>, AppError> {
    let user = require_session_user(&state, &jar).await?;

    let mut finder = ContestRecord::find().filter(contest_records::Column::IsDeleted.eq(false));
    if user.role == "student" {
        let student = Student::find()
            .filter(students::Column::StudentNo.eq(&user.username))
            .filter(students::Column::IsDeleted.eq(false))
            .one(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?
            .ok_or_else(|| AppError::not_found("student not found"))?;
        finder = finder.filter(contest_records::Column::StudentId.eq(student.id));
    } else if user.role != "admin" && user.role != "teacher" && user.role != "reviewer" {
        return Err(AppError::auth("forbidden"));
    } else {
        finder = finder
            .join(JoinType::InnerJoin, contest_records::Relation::Student.def())
            .filter(students::Column::IsDeleted.eq(false));
    }

    if let Some(status) = query.status {
        finder = finder.filter(contest_records::Column::Status.eq(status));
    }

    let records = finder
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let form_fields = load_form_fields(&state, "contest").await?;
    let ids: Vec<Uuid> = records.iter().map(|record| record.id).collect();
    let custom_values = fetch_custom_fields(&state, "contest", &ids, &form_fields).await?;

    let rule_config = load_labor_hour_rules(&state).await?;
    let mut responses = Vec::with_capacity(records.len());
    for record in records {
        let match_status = contest_match_status(&state, &record.contest_name).await?;
        let recommended_hours = compute_recommended_hours(
            rule_config,
            record.contest_category.as_deref(),
            record.contest_level.as_deref(),
            record.contest_role.as_deref(),
        );
        let values = custom_values.get(&record.id).cloned().unwrap_or_default();
        responses.push(model_to_contest_response(
            record,
            &match_status,
            recommended_hours,
            values,
        ));
    }

    Ok(Json(responses))
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

    let record = ContestRecord::find()
        .filter(contest_records::Column::Id.eq(record_id))
        .filter(contest_records::Column::IsDeleted.eq(false))
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
    let rule_config = load_labor_hour_rules(&state).await?;
    let recommended_hours = compute_recommended_hours(
        rule_config,
        model.contest_category.as_deref(),
        model.contest_level.as_deref(),
        model.contest_role.as_deref(),
    );
    let form_fields = load_form_fields(&state, "contest").await?;
    let model_id = model.id;
    let custom_values = fetch_custom_fields(&state, "contest", &[model_id], &form_fields).await?;
    Ok(Json(model_to_contest_response(
        model,
        &match_status,
        recommended_hours,
        custom_values.get(&model_id).cloned().unwrap_or_default(),
    )))
}

fn model_to_contest_response(
    model: contest_records::Model,
    match_status: &str,
    recommended_hours: i32,
    custom_fields: Vec<CustomFieldValueResponse>,
) -> ContestRecordResponse {
    ContestRecordResponse {
        id: model.id,
        student_id: model.student_id,
        contest_name: model.contest_name,
        contest_year: model.contest_year,
        contest_category: model.contest_category,
        contest_level: model.contest_level,
        contest_role: model.contest_role,
        award_level: model.award_level,
        award_date: model.award_date.map(|value| value.to_rfc3339()),
        self_hours: model.self_hours,
        first_review_hours: model.first_review_hours,
        final_review_hours: model.final_review_hours,
        status: model.status,
        rejection_reason: model.rejection_reason,
        match_status: match_status.to_string(),
        recommended_hours,
        custom_fields,
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

fn parse_award_date(value: Option<&str>) -> Result<Option<chrono::DateTime<chrono::Utc>>, AppError> {
    let Some(raw) = value else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return Ok(Some(dt.with_timezone(&Utc)));
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        let dt = date.and_hms_opt(0, 0, 0).ok_or_else(|| AppError::validation("invalid award date"))?;
        return Ok(Some(chrono::DateTime::<Utc>::from_utc(dt, Utc)));
    }
    Err(AppError::validation("invalid award date"))
}

async fn load_form_fields(
    state: &AppState,
    form_type: &str,
) -> Result<Vec<form_fields::Model>, AppError> {
    FormField::find()
        .filter(form_fields::Column::FormType.eq(form_type))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))
}

fn validate_custom_fields(
    fields: &[form_fields::Model],
    payload: &HashMap<String, String>,
) -> Result<(), AppError> {
    let mut field_map = HashMap::new();
    for field in fields {
        field_map.insert(field.field_key.as_str(), field);
    }

    for field in fields {
        if field.required {
            let value = payload.get(&field.field_key);
            if value.is_none() || value.is_some_and(|val| val.trim().is_empty()) {
                return Err(AppError::validation("missing required custom field"));
            }
        }
    }

    for key in payload.keys() {
        if !field_map.contains_key(key.as_str()) {
            return Err(AppError::validation("unknown custom field"));
        }
    }

    Ok(())
}

async fn insert_custom_fields(
    state: &AppState,
    record_type: &str,
    record_id: Uuid,
    fields: &[form_fields::Model],
    payload: &HashMap<String, String>,
) -> Result<(), AppError> {
    let mut field_map = HashMap::new();
    for field in fields {
        field_map.insert(field.field_key.as_str(), field);
    }

    for (key, value) in payload {
        if value.trim().is_empty() {
            continue;
        }
        if let Some(field) = field_map.get(key.as_str()) {
            let value_model = form_field_values::ActiveModel {
                id: Set(Uuid::new_v4()),
                record_type: Set(record_type.to_string()),
                record_id: Set(record_id),
                field_key: Set(field.field_key.clone()),
                value: Set(value.to_string()),
                created_at: Set(Utc::now()),
            };
            form_field_values::Entity::insert(value_model)
                .exec_without_returning(&state.db)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?;
        }
    }

    Ok(())
}

async fn fetch_custom_fields(
    state: &AppState,
    record_type: &str,
    record_ids: &[Uuid],
    fields: &[form_fields::Model],
) -> Result<HashMap<Uuid, Vec<CustomFieldValueResponse>>, AppError> {
    if record_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut label_map = HashMap::new();
    let mut order_map = HashMap::new();
    for field in fields {
        label_map.insert(field.field_key.as_str(), field.label.clone());
        order_map.insert(field.field_key.as_str(), field.order_index);
    }

    let values = FormFieldValue::find()
        .filter(form_field_values::Column::RecordType.eq(record_type))
        .filter(form_field_values::Column::RecordId.is_in(record_ids.iter().cloned()))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut grouped: HashMap<Uuid, Vec<CustomFieldValueResponse>> = HashMap::new();
    for value in values {
        let label = label_map
            .get(value.field_key.as_str())
            .cloned()
            .unwrap_or_else(|| value.field_key.clone());
        grouped
            .entry(value.record_id)
            .or_default()
            .push(CustomFieldValueResponse {
                field_key: value.field_key,
                label,
                value: value.value,
            });
    }

    for list in grouped.values_mut() {
        list.sort_by_key(|item| order_map.get(item.field_key.as_str()).cloned().unwrap_or(0));
    }

    Ok(grouped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

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

    #[test]
    fn ensure_review_permission_allows_expected_roles() {
        let user = crate::entities::users::Model {
            id: Uuid::new_v4(),
            username: "u1".to_string(),
            display_name: "u1".to_string(),
            role: "reviewer".to_string(),
            email: None,
            password_hash: None,
            allow_password_login: false,
            password_updated_at: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        ensure_review_permission(&user, REVIEW_STAGE_FIRST).expect("reviewer allowed");
        assert!(ensure_review_permission(&user, REVIEW_STAGE_FINAL).is_err());
    }

    #[test]
    fn validate_custom_fields_rejects_missing_required_and_unknown() {
        let fields = vec![
            form_fields::Model {
                id: Uuid::new_v4(),
                form_type: "contest".to_string(),
                field_key: "location".to_string(),
                label: "地点".to_string(),
                field_type: "text".to_string(),
                required: true,
                order_index: 1,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            form_fields::Model {
                id: Uuid::new_v4(),
                form_type: "contest".to_string(),
                field_key: "note".to_string(),
                label: "备注".to_string(),
                field_type: "text".to_string(),
                required: false,
                order_index: 2,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        let empty_payload = HashMap::new();
        assert!(validate_custom_fields(&fields, &empty_payload).is_err());

        let mut unknown_payload = HashMap::new();
        unknown_payload.insert("unknown".to_string(), "value".to_string());
        assert!(validate_custom_fields(&fields, &unknown_payload).is_err());

        let mut ok_payload = HashMap::new();
        ok_payload.insert("location".to_string(), "校内".to_string());
        assert!(validate_custom_fields(&fields, &ok_payload).is_ok());
    }

    #[test]
    fn model_to_response_copies_fields() {
        let contest = contest_records::Model {
            id: Uuid::new_v4(),
            student_id: Uuid::new_v4(),
            contest_year: Some(2024),
            contest_category: Some("A".to_string()),
            contest_name: "竞赛".to_string(),
            contest_level: Some("国家级".to_string()),
            contest_role: Some("负责人".to_string()),
            award_level: "一等奖".to_string(),
            award_date: None,
            self_hours: 3,
            first_review_hours: None,
            final_review_hours: None,
            status: STATUS_SUBMITTED.to_string(),
            rejection_reason: None,
            is_deleted: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let contest_resp = model_to_contest_response(contest, "matched", 2, Vec::new());
        assert_eq!(contest_resp.match_status, "matched");
        assert_eq!(contest_resp.contest_name, "竞赛");
    }
}
