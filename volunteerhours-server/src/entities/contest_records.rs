//! 竞赛获奖记录。

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "contest_records")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub student_id: Uuid,
    pub contest_name: String,
    pub award_level: String,
    pub self_hours: i32,
    pub first_review_hours: Option<i32>,
    pub final_review_hours: Option<i32>,
    pub status: String,
    pub rejection_reason: Option<String>,
    pub is_deleted: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::students::Entity",
        from = "Column::StudentId",
        to = "super::students::Column::Id"
    )]
    Student,
}

impl Related<super::students::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Student.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
