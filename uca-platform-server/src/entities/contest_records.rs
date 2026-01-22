//! 竞赛获奖记录。

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "contest_records")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub student_id: Uuid,
    pub contest_year: Option<i32>,
    pub contest_category: Option<String>,
    pub contest_name: String,
    pub contest_level: Option<String>,
    pub contest_role: Option<String>,
    pub award_level: String,
    pub award_date: Option<DateTimeUtc>,
    pub self_hours: i32,
    pub first_review_hours: Option<i32>,
    pub final_review_hours: Option<i32>,
    pub first_reviewer_id: Option<Uuid>,
    pub final_reviewer_id: Option<Uuid>,
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
