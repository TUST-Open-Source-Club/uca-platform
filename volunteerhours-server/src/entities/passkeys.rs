//! Passkey 凭据记录。

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "passkeys")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub credential_id: String,
    pub passkey_json: String,
    pub created_at: DateTimeUtc,
    pub last_used_at: Option<DateTimeUtc>,
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
