//! 邀请注册记录。

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "invites")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub token_hash: String,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub expires_at: DateTimeUtc,
    pub created_at: DateTimeUtc,
    pub used_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
