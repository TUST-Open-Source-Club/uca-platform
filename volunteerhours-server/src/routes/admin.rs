//! 管理员维护接口。

use axum::{extract::State, Json};
use axum_extra::extract::cookie::CookieJar;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
    access::{require_role, require_session_user},
    entities::{
        competition_library, form_fields, CompetitionLibrary, FormField,
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
    let model = competition_library::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(payload.name),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&state.db)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(CompetitionResponse { id: model.id, name: model.name }))
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
    let model = form_fields::ActiveModel {
        id: Set(Uuid::new_v4()),
        form_type: Set(payload.form_type),
        field_key: Set(payload.field_key),
        label: Set(payload.label),
        field_type: Set(payload.field_type),
        required: Set(payload.required),
        order_index: Set(payload.order_index),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&state.db)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(FormFieldResponse {
        id: model.id,
        form_type: model.form_type,
        field_key: model.field_key,
        label: model.label,
        field_type: model.field_type,
        required: model.required,
        order_index: model.order_index,
    }))
}
