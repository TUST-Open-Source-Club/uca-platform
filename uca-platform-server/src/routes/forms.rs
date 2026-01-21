//! 表单配置读取接口。

use axum::{extract::Path, Json};
use axum::extract::State;
use axum_extra::extract::cookie::CookieJar;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    access::require_session_user,
    entities::{form_fields, FormField},
    error::AppError,
    state::AppState,
};

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

/// 按类型读取表单字段。
pub async fn list_form_fields_for_type(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(form_type): Path<String>,
) -> Result<Json<Vec<FormFieldResponse>>, AppError> {
    let _user = require_session_user(&state, &jar).await?;

    let fields = FormField::find()
        .filter(form_fields::Column::FormType.eq(form_type))
        .order_by_asc(form_fields::Column::OrderIndex)
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
