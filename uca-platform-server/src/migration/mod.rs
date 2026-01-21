//! 数据库迁移。

use sea_orm_migration::prelude::*;

mod m20250121_000001_create_auth_tables;
mod m20250121_000002_create_core_tables;
mod m20250210_000003_add_soft_delete;

/// UCA Platform 数据库迁移器。
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250121_000001_create_auth_tables::Migration),
            Box::new(m20250121_000002_create_core_tables::Migration),
            Box::new(m20250210_000003_add_soft_delete::Migration),
        ]
    }
}
