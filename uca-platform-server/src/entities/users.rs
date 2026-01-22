//! 用户账户。

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub allow_password_login: bool,
    pub password_updated_at: Option<DateTimeUtc>,
    pub must_change_password: bool,
    pub is_active: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
