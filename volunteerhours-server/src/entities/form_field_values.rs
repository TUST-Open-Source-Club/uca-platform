//! 自定义表单字段取值。

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "form_field_values")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub record_type: String,
    pub record_id: Uuid,
    pub field_key: String,
    pub value: String,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
