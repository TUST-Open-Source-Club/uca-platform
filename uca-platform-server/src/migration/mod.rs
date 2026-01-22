//! 数据库迁移。

use sea_orm_migration::prelude::*;

mod m20250121_000001_create_auth_tables;
mod m20250121_000002_create_core_tables;
mod m20250210_000003_add_soft_delete;
mod m20250212_000004_auth_invites_and_passwords;
mod m20250215_000005_labor_hours_templates;

/// Labor Hours Platform 数据库迁移器。
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250121_000001_create_auth_tables::Migration),
            Box::new(m20250121_000002_create_core_tables::Migration),
        Box::new(m20250210_000003_add_soft_delete::Migration),
        Box::new(m20250212_000004_auth_invites_and_passwords::Migration),
        Box::new(m20250215_000005_labor_hours_templates::Migration),
        ]
    }
}
