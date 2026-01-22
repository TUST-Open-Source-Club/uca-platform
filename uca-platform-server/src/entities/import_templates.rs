//! 导入模板定义。

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "import_templates")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub template_key: String,
    pub name: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::import_template_fields::Entity")]
    Fields,
}

impl Related<super::import_template_fields::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Fields.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
