//! 自定义表单字段定义。

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "form_fields")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub form_type: String,
    pub field_key: String,
    pub label: String,
    pub field_type: String,
    pub required: bool,
    pub order_index: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
