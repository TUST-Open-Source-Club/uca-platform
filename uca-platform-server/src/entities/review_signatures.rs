//! 审核签名记录。

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "review_signatures")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub record_type: String,
    pub record_id: Uuid,
    pub reviewer_user_id: Uuid,
    pub stage: String,
    pub signature_path: String,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
