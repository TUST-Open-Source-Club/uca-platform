//! 认证重置令牌。

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "auth_resets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub token_hash: String,
    pub user_id: Uuid,
    pub purpose: String,
    pub expires_at: DateTimeUtc,
    pub created_at: DateTimeUtc,
    pub used_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
