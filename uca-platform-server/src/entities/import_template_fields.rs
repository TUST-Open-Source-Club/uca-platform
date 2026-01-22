//! 导入模板字段映射。

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "import_template_fields")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub template_id: Uuid,
    pub field_key: String,
    pub label: String,
    pub column_title: String,
    pub required: bool,
    pub order_index: i32,
    pub description: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::import_templates::Entity",
        from = "Column::TemplateId",
        to = "super::import_templates::Column::Id"
    )]
    Template,
}

impl Related<super::import_templates::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Template.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
