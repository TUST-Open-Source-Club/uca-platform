//! 劳动教育学时规则。

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "labor_hour_rules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub base_hours_a: i32,
    pub base_hours_b: i32,
    pub national_leader_hours: i32,
    pub national_member_hours: i32,
    pub provincial_leader_hours: i32,
    pub provincial_member_hours: i32,
    pub school_leader_hours: i32,
    pub school_member_hours: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
