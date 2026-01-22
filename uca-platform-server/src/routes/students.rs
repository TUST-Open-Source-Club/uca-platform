//! 学生名单接口。

use axum::{extract::{State, Multipart, Path}, Json};
use axum_extra::extract::cookie::CookieJar;
use calamine::{Data, Reader};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use uuid::Uuid;
use validator::Validate;

use crate::{
    access::{require_role, require_session_user},
    auth::hash_password,
    entities::{students, users, Student, User},
    error::AppError,
    templates::{build_header_index, read_cell_by_index},
    state::AppState,
};

/// 学生列表响应。
#[derive(Debug, Serialize)]
pub struct StudentResponse {
    /// 学生 ID。
    pub id: Uuid,
    /// 学号。
    pub student_no: String,
    /// 姓名。
    pub name: String,
    /// 性别。
    pub gender: String,
    /// 院系。
    pub department: String,
    /// 专业。
    pub major: String,
    /// 班级。
    pub class_name: String,
    /// 手机号。
    pub phone: String,
}

impl From<students::Model> for StudentResponse {
    fn from(model: students::Model) -> Self {
        Self {
            id: model.id,
            student_no: model.student_no,
            name: model.name,
            gender: model.gender,
            department: model.department,
            major: model.major,
            class_name: model.class_name,
            phone: model.phone,
        }
    }
}

/// 新建学生请求。
#[derive(Debug, Deserialize, Validate)]
pub struct CreateStudentRequest {
    /// 学号。
    #[validate(length(min = 4, max = 32))]
    pub student_no: String,
    /// 姓名。
    #[validate(length(min = 1, max = 64))]
    pub name: String,
    /// 性别。
    #[validate(length(min = 1, max = 8))]
    pub gender: String,
    /// 院系。
    #[validate(length(min = 1, max = 64))]
    pub department: String,
    /// 专业。
    #[validate(length(min = 1, max = 64))]
    pub major: String,
    /// 班级。
    #[validate(length(min = 1, max = 64))]
    pub class_name: String,
    /// 手机号。
    #[validate(length(min = 6, max = 32))]
    pub phone: String,
}

/// 更新学生请求。
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateStudentRequest {
    /// 姓名。
    #[validate(length(min = 1, max = 64))]
    pub name: String,
    /// 性别。
    #[validate(length(min = 1, max = 8))]
    pub gender: String,
    /// 院系。
    #[validate(length(min = 1, max = 64))]
    pub department: String,
    /// 专业。
    #[validate(length(min = 1, max = 64))]
    pub major: String,
    /// 班级。
    #[validate(length(min = 1, max = 64))]
    pub class_name: String,
    /// 手机号。
    #[validate(length(min = 6, max = 32))]
    pub phone: String,
}

/// 创建学生（仅管理员）。
pub async fn create_student(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<CreateStudentRequest>,
) -> Result<Json<StudentResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    payload
        .validate()
        .map_err(|_| AppError::validation("invalid student payload"))?;

    let exists = Student::find()
        .filter(students::Column::StudentNo.eq(&payload.student_no))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    if let Some(existing) = exists {
        if existing.is_deleted {
            let mut active: students::ActiveModel = existing.into();
            active.name = Set(payload.name.clone());
            active.gender = Set(payload.gender.clone());
            active.department = Set(payload.department.clone());
            active.major = Set(payload.major.clone());
            active.class_name = Set(payload.class_name.clone());
            active.phone = Set(payload.phone.clone());
            active.is_deleted = Set(false);
            active.updated_at = Set(Utc::now());
            let model = active
                .update(&state.db)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?;
            upsert_student_user(&state.db, &payload.student_no, &payload.name, true).await?;
            return Ok(Json(StudentResponse::from(model)));
        }
        return Err(AppError::bad_request("student number exists"));
    }

    let now = Utc::now();
    let id = Uuid::new_v4();
    let model = students::ActiveModel {
        id: Set(id),
        student_no: Set(payload.student_no.clone()),
        name: Set(payload.name.clone()),
        gender: Set(payload.gender.clone()),
        department: Set(payload.department.clone()),
        major: Set(payload.major.clone()),
        class_name: Set(payload.class_name.clone()),
        phone: Set(payload.phone.clone()),
        is_deleted: Set(false),
        created_at: Set(now),
        updated_at: Set(now),
    };
    students::Entity::insert(model)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    upsert_student_user(&state.db, &payload.student_no, &payload.name, true).await?;

    let model = students::Model {
        id,
        student_no: payload.student_no,
        name: payload.name,
        gender: payload.gender,
        department: payload.department,
        major: payload.major,
        class_name: payload.class_name,
        phone: payload.phone,
        is_deleted: false,
        created_at: now,
        updated_at: now,
    };
    Ok(Json(StudentResponse::from(model)))
}

/// 更新学生信息（仅管理员）。
pub async fn update_student(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(student_no): Path<String>,
    Json(payload): Json<UpdateStudentRequest>,
) -> Result<Json<StudentResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    payload
        .validate()
        .map_err(|_| AppError::validation("invalid student payload"))?;

    let student = Student::find()
        .filter(students::Column::StudentNo.eq(&student_no))
        .filter(students::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("student not found"))?;

    let mut active: students::ActiveModel = student.into();
    active.name = Set(payload.name.clone());
    active.gender = Set(payload.gender.clone());
    active.department = Set(payload.department.clone());
    active.major = Set(payload.major.clone());
    active.class_name = Set(payload.class_name.clone());
    active.phone = Set(payload.phone.clone());
    active.updated_at = Set(Utc::now());
    let model = active
        .update(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    upsert_student_user(&state.db, &student_no, &payload.name, true).await?;

    Ok(Json(StudentResponse::from(model)))
}

/// 学生筛选查询。
#[derive(Debug, Deserialize)]
pub struct StudentQuery {
    /// 院系筛选（可选）。
    pub department: Option<String>,
    /// 专业筛选（可选）。
    pub major: Option<String>,
    /// 班级筛选（可选）。
    pub class_name: Option<String>,
    /// 学号或姓名关键词（可选）。
    pub keyword: Option<String>,
}

/// 学生列表（带筛选条件）。
pub async fn list_students(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(query): Json<StudentQuery>,
) -> Result<Json<Vec<StudentResponse>>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    if user.role != "admin" && user.role != "teacher" && user.role != "reviewer" {
        return Err(AppError::auth("forbidden"));
    }

    let mut finder = Student::find().filter(students::Column::IsDeleted.eq(false));
    if let Some(value) = query.department {
        finder = finder.filter(students::Column::Department.eq(value));
    }
    if let Some(value) = query.major {
        finder = finder.filter(students::Column::Major.eq(value));
    }
    if let Some(value) = query.class_name {
        finder = finder.filter(students::Column::ClassName.eq(value));
    }
    if let Some(keyword) = query.keyword {
        let condition = Condition::any()
            .add(students::Column::StudentNo.contains(&keyword))
            .add(students::Column::Name.contains(&keyword));
        finder = finder.filter(condition);
    }

    let results = finder
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(results.into_iter().map(StudentResponse::from).collect()))
}

/// 从 Excel 导入学生（仅管理员）。
pub async fn import_students(
    State(state): State<AppState>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let (file_bytes, fields) = read_upload_payload(&mut multipart).await?;
    let field_map = fields
        .get("field_map")
        .map(|value| serde_json::from_str::<HashMap<String, String>>(value))
        .transpose()
        .map_err(|_| AppError::bad_request("invalid field_map"))?;
    let allow_login = fields
        .get("allow_login")
        .map(|value| value == "true" || value == "1")
        .unwrap_or(false);
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
    let base_index = build_student_field_map(&header_index, field_map.as_ref())?;

    let transaction = state
        .db
        .begin()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut inserted = 0usize;
    let mut updated = 0usize;

    for row in range.rows().skip(1) {
        let student_no = read_cell_by_index_opt(base_index.get("student_no"), row);
        let name = read_cell_by_index_opt(base_index.get("name"), row);
        let gender = read_cell_by_index_opt(base_index.get("gender"), row);
        let department = read_cell_by_index_opt(base_index.get("department"), row);
        let major = read_cell_by_index_opt(base_index.get("major"), row);
        let class_name = read_cell_by_index_opt(base_index.get("class_name"), row);
        let phone = read_cell_by_index_opt(base_index.get("phone"), row);

        if student_no.is_empty() || name.is_empty() {
            continue;
        }

        let existing = Student::find()
            .filter(students::Column::StudentNo.eq(&student_no))
            .one(&transaction)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;

        let now = Utc::now();
        if let Some(record) = existing {
            let mut active: students::ActiveModel = record.into();
            active.name = Set(name.clone());
            active.gender = Set(gender);
            active.department = Set(department);
            active.major = Set(major);
            active.class_name = Set(class_name);
            active.phone = Set(phone);
            active.updated_at = Set(now);
            active.is_deleted = Set(false);
            active
                .update(&transaction)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?;
            upsert_student_user(&transaction, &student_no, &name, allow_login).await?;
            updated += 1;
        } else {
            let model = students::ActiveModel {
                id: Set(Uuid::new_v4()),
                student_no: Set(student_no.clone()),
                name: Set(name.clone()),
                gender: Set(gender),
                department: Set(department),
                major: Set(major),
                class_name: Set(class_name),
                phone: Set(phone),
                is_deleted: Set(false),
                created_at: Set(now),
                updated_at: Set(now),
            };
            students::Entity::insert(model)
                .exec_without_returning(&transaction)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?;
            upsert_student_user(&transaction, &student_no, &name, allow_login).await?;
            inserted += 1;
        }
    }

    transaction
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(serde_json::json!({
        "inserted": inserted,
        "updated": updated
    })))
}

fn read_cell_by_index_opt(index: Option<&usize>, row: &[calamine::Data]) -> String {
    let idx = match index {
        Some(value) => *value,
        None => return String::new(),
    };
    read_cell_by_index(idx, row)
}

async fn read_upload_payload(
    multipart: &mut Multipart,
) -> Result<(Vec<u8>, HashMap<String, String>), AppError> {
    let mut file_bytes = None;
    let mut fields = HashMap::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::bad_request("invalid multipart"))?
    {
        let name = field.name().map(|value| value.to_string());
        match name.as_deref() {
            Some("file") => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|_| AppError::bad_request("failed to read file"))?;
                file_bytes = Some(bytes.to_vec());
            }
            Some(key) => {
                let value = field
                    .text()
                    .await
                    .map_err(|_| AppError::bad_request("failed to read field"))?;
                fields.insert(key.to_string(), value);
            }
            None => {}
        }
    }
    let file_bytes = file_bytes.ok_or_else(|| AppError::bad_request("file field required"))?;
    Ok((file_bytes, fields))
}

fn build_student_field_map(
    header_index: &HashMap<String, usize>,
    field_map: Option<&HashMap<String, String>>,
) -> Result<HashMap<String, usize>, AppError> {
    let mut result = HashMap::new();
    for (key, candidates, required) in [
        ("student_no", &["学号", "student_no"][..], true),
        ("name", &["姓名", "name"][..], true),
        ("gender", &["性别", "gender"][..], false),
        ("department", &["院系", "department"][..], false),
        ("major", &["专业", "major"][..], false),
        ("class_name", &["班级", "class_name"][..], false),
        ("phone", &["手机号", "phone"][..], false),
    ] {
        let override_value = field_map.and_then(|map| map.get(key).map(|value| value.as_str()));
        let idx = resolve_column_index(header_index, override_value, candidates);
        if required && idx.is_none() {
            return Err(AppError::bad_request("missing required header"));
        }
        if let Some(idx) = idx {
            result.insert(key.to_string(), idx);
        }
    }
    Ok(result)
}

fn resolve_column_index(
    header_index: &HashMap<String, usize>,
    column: Option<&str>,
    fallback: &[&str],
) -> Option<usize> {
    if let Some(value) = column {
        let trimmed = value.trim();
        if let Some(idx) = parse_column_reference(trimmed) {
            return Some(idx);
        }
        if let Some(idx) = header_index.get(trimmed) {
            return Some(*idx);
        }
    }
    fallback.iter().find_map(|key| header_index.get(*key).copied())
}

fn parse_column_reference(value: &str) -> Option<usize> {
    if value.is_empty() {
        return None;
    }
    if value.chars().all(|ch| ch.is_ascii_digit()) {
        let number = value.parse::<usize>().ok()?;
        return number.checked_sub(1);
    }
    if value.chars().all(|ch| ch.is_ascii_alphabetic()) {
        let mut index = 0usize;
        for ch in value.chars() {
            let upper = ch.to_ascii_uppercase();
            let offset = upper as u8;
            if offset < b'A' || offset > b'Z' {
                return None;
            }
            index = index * 26 + (offset - b'A' + 1) as usize;
        }
        return index.checked_sub(1);
    }
    None
}

async fn upsert_student_user<C>(
    db: &C,
    student_no: &str,
    name: &str,
    allow_login: bool,
) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    let default_password = format!("st{student_no}");
    let default_hash = hash_password(&default_password)?;
    let now = Utc::now();
    if let Some(existing) = User::find()
        .filter(users::Column::Username.eq(student_no))
        .one(db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
    {
        let missing_password = existing.password_hash.is_none();
        let mut active: users::ActiveModel = existing.into();
        active.display_name = Set(name.to_string());
        active.role = Set("student".to_string());
        if missing_password {
            active.password_hash = Set(Some(default_hash));
        }
        active.allow_password_login = Set(allow_login);
        active.updated_at = Set(now);
        active
            .update(db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
        return Ok(());
    }

    let model = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(student_no.to_string()),
        display_name: Set(name.to_string()),
        role: Set("student".to_string()),
        email: Set(None),
        password_hash: Set(Some(default_hash)),
        allow_password_login: Set(allow_login),
        password_updated_at: Set(Some(now)),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    };
    users::Entity::insert(model)
        .exec_without_returning(db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(())
}

fn read_cell(index: &std::collections::HashMap<String, usize>, key: &str, row: &[Data]) -> String {
    let idx = match index.get(key) {
        Some(idx) => *idx,
        None => return String::new(),
    };
    row.get(idx)
        .map(|cell| cell.to_string().trim().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_cell_returns_trimmed_string() {
        let mut index = std::collections::HashMap::new();
        index.insert("学号".to_string(), 0);
        let row = vec![Data::String(" 2023001 ".to_string())];
        assert_eq!(read_cell(&index, "学号", &row), "2023001");
    }

    #[test]
    fn read_cell_returns_empty_on_missing_header() {
        let index = std::collections::HashMap::new();
        let row = vec![Data::String(" 2023001 ".to_string())];
        assert_eq!(read_cell(&index, "学号", &row), "");
    }
}
