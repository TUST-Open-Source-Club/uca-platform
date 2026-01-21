//! 管理员维护接口。

use axum::{extract::{State, Multipart}, Json};
use axum_extra::extract::cookie::CookieJar;
use calamine::{Data, Reader};
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use uuid::Uuid;
use validator::Validate;

use crate::{
    access::{require_role, require_session_user},
    entities::{
        competition_library, form_field_values, form_fields, contest_records, students,
        volunteer_records, CompetitionLibrary, FormField, Student,
    },
    error::AppError,
    state::AppState,
};

/// 竞赛库新增请求。
#[derive(Debug, Deserialize, Validate)]
pub struct CreateCompetitionRequest {
    /// 竞赛名称。
    #[validate(length(min = 1, max = 200))]
    pub name: String,
}

/// 竞赛库响应。
#[derive(Debug, Serialize)]
pub struct CompetitionResponse {
    /// 记录 ID。
    pub id: Uuid,
    /// 竞赛名称。
    pub name: String,
}

const COMPETITION_HEADER: [&str; 2] = ["竞赛名称", "name"];
const VOLUNTEER_BASE_HEADERS: [(&str, &[&str]); 6] = [
    ("student_no", &["学号", "student_no"]),
    ("title", &["标题", "志愿服务标题", "title"]),
    ("description", &["描述", "志愿服务内容", "description"]),
    ("self_hours", &["自评学时", "self_hours"]),
    ("first_review_hours", &["初审学时", "first_review_hours"]),
    ("final_review_hours", &["复审学时", "final_review_hours"]),
];
const CONTEST_BASE_HEADERS: [(&str, &[&str]); 6] = [
    ("student_no", &["学号", "student_no"]),
    ("contest_name", &["竞赛名称", "contest_name"]),
    ("award_level", &["获奖等级", "award_level"]),
    ("self_hours", &["自评学时", "self_hours"]),
    ("first_review_hours", &["初审学时", "first_review_hours"]),
    ("final_review_hours", &["复审学时", "final_review_hours"]),
];
const STATUS_HEADERS: [&str; 2] = ["审核状态", "status"];
const REJECTION_HEADERS: [&str; 2] = ["不通过原因", "rejection_reason"];

/// 查询竞赛库。
pub async fn list_competitions(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<CompetitionResponse>>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let items = CompetitionLibrary::find()
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(
        items
            .into_iter()
            .map(|item| CompetitionResponse { id: item.id, name: item.name })
            .collect(),
    ))
}

/// 竞赛库公开读取（需登录）。
pub async fn list_competitions_public(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<CompetitionResponse>>, AppError> {
    let _user = require_session_user(&state, &jar).await?;

    let items = CompetitionLibrary::find()
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(
        items
            .into_iter()
            .map(|item| CompetitionResponse { id: item.id, name: item.name })
            .collect(),
    ))
}

/// 新增竞赛名称。
pub async fn create_competition(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<CreateCompetitionRequest>,
) -> Result<Json<CompetitionResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;
    payload
        .validate()
        .map_err(|_| AppError::validation("invalid competition payload"))?;

    let exists = CompetitionLibrary::find()
        .filter(competition_library::Column::Name.eq(&payload.name))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    if exists.is_some() {
        return Err(AppError::bad_request("competition exists"));
    }

    let now = Utc::now();
    let id = Uuid::new_v4();
    let name = payload.name;
    let model = competition_library::ActiveModel {
        id: Set(id),
        name: Set(name.clone()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    competition_library::Entity::insert(model)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(CompetitionResponse { id, name }))
}

/// 从 Excel 导入竞赛名称（仅管理员）。
pub async fn import_competitions(
    State(state): State<AppState>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let file_bytes = read_upload_bytes(&mut multipart).await?;
    let mut workbook = calamine::Xlsx::new(Cursor::new(file_bytes))
        .map_err(|_| AppError::bad_request("invalid xlsx file"))?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| AppError::bad_request("xlsx has no sheets"))?;
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|_| AppError::bad_request("failed to read worksheet"))?;

    let header_index = build_header_index(range.rows().next());
    let name_idx = find_header_index(&header_index, &COMPETITION_HEADER)
        .ok_or_else(|| AppError::bad_request("missing competition header"))?;

    let mut inserted = 0usize;
    let mut skipped = 0usize;
    for row in range.rows().skip(1) {
        let name = read_cell_by_index(name_idx, row);
        if name.is_empty() {
            continue;
        }
        let exists = CompetitionLibrary::find()
            .filter(competition_library::Column::Name.eq(&name))
            .one(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
        if exists.is_some() {
            skipped += 1;
            continue;
        }
        let now = Utc::now();
        let model = competition_library::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(name),
            created_at: Set(now),
            updated_at: Set(now),
        };
        competition_library::Entity::insert(model)
            .exec_without_returning(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
        inserted += 1;
    }

    Ok(Json(serde_json::json!({ "inserted": inserted, "skipped": skipped })))
}

/// 表单字段新增请求。
#[derive(Debug, Deserialize, Validate)]
pub struct CreateFormFieldRequest {
    /// 表单类型。
    #[validate(length(min = 1, max = 32))]
    pub form_type: String,
    /// 字段 key。
    #[validate(length(min = 1, max = 64))]
    pub field_key: String,
    /// 字段标签。
    #[validate(length(min = 1, max = 64))]
    pub label: String,
    /// 字段类型。
    #[validate(length(min = 1, max = 32))]
    pub field_type: String,
    /// 是否必填。
    pub required: bool,
    /// 排序序号。
    pub order_index: i32,
}

/// 表单字段响应。
#[derive(Debug, Serialize)]
pub struct FormFieldResponse {
    /// 记录 ID。
    pub id: Uuid,
    /// 表单类型。
    pub form_type: String,
    /// 字段 key。
    pub field_key: String,
    /// 字段标签。
    pub label: String,
    /// 字段类型。
    pub field_type: String,
    /// 是否必填。
    pub required: bool,
    /// 排序序号。
    pub order_index: i32,
}

/// 查询表单字段。
pub async fn list_form_fields(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<FormFieldResponse>>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let fields = FormField::find()
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(
        fields
            .into_iter()
            .map(|field| FormFieldResponse {
                id: field.id,
                form_type: field.form_type,
                field_key: field.field_key,
                label: field.label,
                field_type: field.field_type,
                required: field.required,
                order_index: field.order_index,
            })
            .collect(),
    ))
}

/// 新增表单字段。
pub async fn create_form_field(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<CreateFormFieldRequest>,
) -> Result<Json<FormFieldResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;
    payload
        .validate()
        .map_err(|_| AppError::validation("invalid form field payload"))?;

    let now = Utc::now();
    let id = Uuid::new_v4();
    let model = form_fields::ActiveModel {
        id: Set(id),
        form_type: Set(payload.form_type.clone()),
        field_key: Set(payload.field_key.clone()),
        label: Set(payload.label.clone()),
        field_type: Set(payload.field_type.clone()),
        required: Set(payload.required),
        order_index: Set(payload.order_index),
        created_at: Set(now),
        updated_at: Set(now),
    };
    form_fields::Entity::insert(model)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(FormFieldResponse {
        id,
        form_type: payload.form_type,
        field_key: payload.field_key,
        label: payload.label,
        field_type: payload.field_type,
        required: payload.required,
        order_index: payload.order_index,
    }))
}

/// 批量导入志愿服务记录（仅管理员）。
pub async fn import_volunteer_records(
    State(state): State<AppState>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let file_bytes = read_upload_bytes(&mut multipart).await?;
    let mut workbook = calamine::Xlsx::new(Cursor::new(file_bytes))
        .map_err(|_| AppError::bad_request("invalid xlsx file"))?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| AppError::bad_request("xlsx has no sheets"))?;
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|_| AppError::bad_request("failed to read worksheet"))?;

    let header_index = build_header_index(range.rows().next());
    let base_index = map_base_indices(&header_index, &VOLUNTEER_BASE_HEADERS);
    ensure_required_headers(&base_index, &["student_no", "title", "description", "self_hours"])?;
    let status_idx = find_header_index(&header_index, &STATUS_HEADERS);
    let rejection_idx = find_header_index(&header_index, &REJECTION_HEADERS);

    let field_map = load_form_field_map(&state, "volunteer").await?;

    let transaction = state
        .db
        .begin()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut inserted = 0usize;
    let mut skipped = 0usize;
    for row in range.rows().skip(1) {
        let student_no = read_cell_by_index_opt(base_index.get("student_no"), row);
        if student_no.is_empty() {
            skipped += 1;
            continue;
        }

        let student = Student::find()
            .filter(students::Column::StudentNo.eq(&student_no))
            .one(&transaction)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
        let student = match student {
            Some(student) => student,
            None => {
                skipped += 1;
                continue;
            }
        };

        let title = read_cell_by_index_opt(base_index.get("title"), row);
        let description = read_cell_by_index_opt(base_index.get("description"), row);
        let self_hours = parse_hours(read_cell_by_index_opt(base_index.get("self_hours"), row));
        if title.is_empty() || description.is_empty() || self_hours.is_none() {
            skipped += 1;
            continue;
        }

        let first_review = parse_hours(read_cell_by_index_opt(base_index.get("first_review_hours"), row));
        let final_review = parse_hours(read_cell_by_index_opt(base_index.get("final_review_hours"), row));
        let status_value = read_cell_by_index_opt(status_idx.as_ref(), row);
        let rejection = read_cell_by_index_opt(rejection_idx.as_ref(), row);
        let status = resolve_status(&status_value, first_review, final_review);

        let now = Utc::now();
        let record_id = Uuid::new_v4();
        let model = volunteer_records::ActiveModel {
            id: Set(record_id),
            student_id: Set(student.id),
            title: Set(title),
            description: Set(description),
            self_hours: Set(self_hours.unwrap_or(0)),
            first_review_hours: Set(first_review),
            final_review_hours: Set(final_review),
            status: Set(status),
            rejection_reason: Set(if rejection.is_empty() { None } else { Some(rejection) }),
            created_at: Set(now),
            updated_at: Set(now),
        };
        volunteer_records::Entity::insert(model)
            .exec_without_returning(&transaction)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;

        let reserved_headers = collect_reserved_headers(
            &header_index,
            &VOLUNTEER_BASE_HEADERS,
            &STATUS_HEADERS,
            &REJECTION_HEADERS,
        );
        insert_custom_fields(
            &transaction,
            "volunteer",
            record_id,
            row,
            &header_index,
            &field_map,
            &reserved_headers,
        )
        .await?;
        inserted += 1;
    }

    transaction
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(serde_json::json!({ "inserted": inserted, "skipped": skipped })))
}

/// 批量导入竞赛记录（仅管理员）。
pub async fn import_contest_records(
    State(state): State<AppState>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let file_bytes = read_upload_bytes(&mut multipart).await?;
    let mut workbook = calamine::Xlsx::new(Cursor::new(file_bytes))
        .map_err(|_| AppError::bad_request("invalid xlsx file"))?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| AppError::bad_request("xlsx has no sheets"))?;
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|_| AppError::bad_request("failed to read worksheet"))?;

    let header_index = build_header_index(range.rows().next());
    let base_index = map_base_indices(&header_index, &CONTEST_BASE_HEADERS);
    ensure_required_headers(&base_index, &["student_no", "contest_name", "award_level", "self_hours"])?;
    let status_idx = find_header_index(&header_index, &STATUS_HEADERS);
    let rejection_idx = find_header_index(&header_index, &REJECTION_HEADERS);

    let field_map = load_form_field_map(&state, "contest").await?;

    let transaction = state
        .db
        .begin()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut inserted = 0usize;
    let mut skipped = 0usize;
    for row in range.rows().skip(1) {
        let student_no = read_cell_by_index_opt(base_index.get("student_no"), row);
        if student_no.is_empty() {
            skipped += 1;
            continue;
        }

        let student = Student::find()
            .filter(students::Column::StudentNo.eq(&student_no))
            .one(&transaction)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
        let student = match student {
            Some(student) => student,
            None => {
                skipped += 1;
                continue;
            }
        };

        let contest_name = read_cell_by_index_opt(base_index.get("contest_name"), row);
        let award_level = read_cell_by_index_opt(base_index.get("award_level"), row);
        let self_hours = parse_hours(read_cell_by_index_opt(base_index.get("self_hours"), row));
        if contest_name.is_empty() || award_level.is_empty() || self_hours.is_none() {
            skipped += 1;
            continue;
        }

        let first_review = parse_hours(read_cell_by_index_opt(base_index.get("first_review_hours"), row));
        let final_review = parse_hours(read_cell_by_index_opt(base_index.get("final_review_hours"), row));
        let status_value = read_cell_by_index_opt(status_idx.as_ref(), row);
        let rejection = read_cell_by_index_opt(rejection_idx.as_ref(), row);
        let status = resolve_status(&status_value, first_review, final_review);

        let now = Utc::now();
        let record_id = Uuid::new_v4();
        let model = contest_records::ActiveModel {
            id: Set(record_id),
            student_id: Set(student.id),
            contest_name: Set(contest_name),
            award_level: Set(award_level),
            self_hours: Set(self_hours.unwrap_or(0)),
            first_review_hours: Set(first_review),
            final_review_hours: Set(final_review),
            status: Set(status),
            rejection_reason: Set(if rejection.is_empty() { None } else { Some(rejection) }),
            created_at: Set(now),
            updated_at: Set(now),
        };
        contest_records::Entity::insert(model)
            .exec_without_returning(&transaction)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;

        let reserved_headers = collect_reserved_headers(
            &header_index,
            &CONTEST_BASE_HEADERS,
            &STATUS_HEADERS,
            &REJECTION_HEADERS,
        );
        insert_custom_fields(
            &transaction,
            "contest",
            record_id,
            row,
            &header_index,
            &field_map,
            &reserved_headers,
        )
        .await?;
        inserted += 1;
    }

    transaction
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(serde_json::json!({ "inserted": inserted, "skipped": skipped })))
}

async fn read_upload_bytes(multipart: &mut Multipart) -> Result<Vec<u8>, AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::bad_request("invalid multipart"))?
    {
        if field.name() == Some("file") {
            let bytes = field
                .bytes()
                .await
                .map_err(|_| AppError::bad_request("failed to read file"))?;
            return Ok(bytes.to_vec());
        }
    }
    Err(AppError::bad_request("file field required"))
}

fn build_header_index(header_row: Option<&[Data]>) -> HashMap<String, usize> {
    let mut header_index = HashMap::new();
    if let Some(header_row) = header_row {
        for (idx, cell) in header_row.iter().enumerate() {
            let trimmed = cell.to_string().trim().to_string();
            if !trimmed.is_empty() {
                header_index.insert(trimmed, idx);
            }
        }
    }
    header_index
}

fn find_header_index(
    header_index: &HashMap<String, usize>,
    candidates: &[&str],
) -> Option<usize> {
    candidates.iter().find_map(|key| header_index.get(*key).cloned())
}

fn map_base_indices(
    header_index: &HashMap<String, usize>,
    candidates: &[(&str, &[&str])],
) -> HashMap<String, usize> {
    let mut result = HashMap::new();
    for (key, headers) in candidates {
        if let Some(idx) = find_header_index(header_index, headers) {
            result.insert(key.to_string(), idx);
        }
    }
    result
}

fn ensure_required_headers(
    base_index: &HashMap<String, usize>,
    required: &[&str],
) -> Result<(), AppError> {
    for key in required {
        if !base_index.contains_key(*key) {
            return Err(AppError::bad_request("missing required header"));
        }
    }
    Ok(())
}

fn read_cell_by_index_opt(index: Option<&usize>, row: &[Data]) -> String {
    let idx = match index {
        Some(value) => *value,
        None => return String::new(),
    };
    read_cell_by_index(idx, row)
}

fn read_cell_by_index(idx: usize, row: &[Data]) -> String {
    row.get(idx)
        .map(|cell| cell.to_string().trim().to_string())
        .unwrap_or_default()
}

fn parse_hours(value: String) -> Option<i32> {
    if value.is_empty() {
        return None;
    }
    value.parse::<f32>().ok().map(|num| num.round() as i32)
}

fn resolve_status(status_value: &str, first_review: Option<i32>, final_review: Option<i32>) -> String {
    if status_value == "不通过" || status_value == "rejected" {
        return "rejected".to_string();
    }
    if status_value == "已复审" || status_value == "final_reviewed" || final_review.is_some() {
        return "final_reviewed".to_string();
    }
    if status_value == "已初审" || status_value == "first_reviewed" || first_review.is_some() {
        return "first_reviewed".to_string();
    }
    "submitted".to_string()
}

async fn load_form_field_map(
    state: &AppState,
    form_type: &str,
) -> Result<HashMap<String, form_fields::Model>, AppError> {
    let fields = FormField::find()
        .filter(form_fields::Column::FormType.eq(form_type))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut map = HashMap::new();
    for field in fields {
        map.insert(field.field_key.clone(), field.clone());
        map.insert(field.label.clone(), field);
    }
    Ok(map)
}

fn collect_reserved_headers(
    header_index: &HashMap<String, usize>,
    base_headers: &[(&str, &[&str])],
    status_headers: &[&str],
    rejection_headers: &[&str],
) -> Vec<String> {
    let mut reserved = Vec::new();
    for (_, headers) in base_headers {
        for header in *headers {
            if header_index.contains_key(*header) {
                reserved.push((*header).to_string());
            }
        }
    }
    for header in status_headers {
        if header_index.contains_key(*header) {
            reserved.push((*header).to_string());
        }
    }
    for header in rejection_headers {
        if header_index.contains_key(*header) {
            reserved.push((*header).to_string());
        }
    }
    reserved
}

async fn insert_custom_fields(
    txn: &sea_orm::DatabaseTransaction,
    record_type: &str,
    record_id: Uuid,
    row: &[Data],
    header_index: &HashMap<String, usize>,
    field_map: &HashMap<String, form_fields::Model>,
    reserved_headers: &[String],
) -> Result<(), AppError> {
    for (header, idx) in header_index {
        if reserved_headers.contains(header) {
            continue;
        }
        if let Some(field) = field_map.get(header) {
            let value = read_cell_by_index(*idx, row);
            if value.is_empty() {
                continue;
            }
            let value_model = form_field_values::ActiveModel {
                id: Set(Uuid::new_v4()),
                record_type: Set(record_type.to_string()),
                record_id: Set(record_id),
                field_key: Set(field.field_key.clone()),
                value: Set(value),
                created_at: Set(Utc::now()),
            };
            form_field_values::Entity::insert(value_model)
                .exec_without_returning(txn)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::Data;

    #[test]
    fn build_header_index_trims_and_ignores_empty() {
        let row = vec![
            Data::String(" 学号 ".to_string()),
            Data::String("".to_string()),
            Data::String("姓名".to_string()),
        ];
        let index = build_header_index(Some(&row));
        assert_eq!(index.get("学号").copied(), Some(0));
        assert_eq!(index.get("姓名").copied(), Some(2));
        assert!(!index.contains_key(""));
    }

    #[test]
    fn map_base_indices_resolves_candidates() {
        let mut header_index = HashMap::new();
        header_index.insert("学号".to_string(), 0);
        header_index.insert("标题".to_string(), 1);
        let result = map_base_indices(&header_index, &[("student_no", &["学号"]), ("title", &["标题"])]);
        assert_eq!(result.get("student_no").copied(), Some(0));
        assert_eq!(result.get("title").copied(), Some(1));
    }

    #[test]
    fn ensure_required_headers_detects_missing() {
        let mut base = HashMap::new();
        base.insert("student_no".to_string(), 0);
        let result = ensure_required_headers(&base, &["student_no", "title"]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_hours_handles_rounding() {
        assert_eq!(parse_hours("1.6".to_string()), Some(2));
        assert_eq!(parse_hours("2".to_string()), Some(2));
        assert_eq!(parse_hours("".to_string()), None);
    }

    #[test]
    fn resolve_status_prefers_rejection_then_reviews() {
        assert_eq!(resolve_status("不通过", None, None), "rejected");
        assert_eq!(resolve_status("已复审", None, None), "final_reviewed");
        assert_eq!(resolve_status("", Some(1), None), "first_reviewed");
        assert_eq!(resolve_status("", None, None), "submitted");
    }

    #[test]
    fn collect_reserved_headers_includes_known() {
        let mut index = HashMap::new();
        index.insert("学号".to_string(), 0);
        index.insert("审核状态".to_string(), 1);
        index.insert("备注".to_string(), 2);
        let reserved = collect_reserved_headers(
            &index,
            &[("student_no", &["学号"])],
            &["审核状态"],
            &["备注"],
        );
        assert!(reserved.contains(&"学号".to_string()));
        assert!(reserved.contains(&"审核状态".to_string()));
        assert!(reserved.contains(&"备注".to_string()));
    }
}
