//! 增加软删除标记字段。

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Students::Table)
                    .add_column(
                        ColumnDef::new(Students::IsDeleted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(VolunteerRecords::Table)
                    .add_column(
                        ColumnDef::new(VolunteerRecords::IsDeleted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .add_column(
                        ColumnDef::new(ContestRecords::IsDeleted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .drop_column(ContestRecords::IsDeleted)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(VolunteerRecords::Table)
                    .drop_column(VolunteerRecords::IsDeleted)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Students::Table)
                    .drop_column(Students::IsDeleted)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Students {
    Table,
    IsDeleted,
}

#[derive(Iden)]
enum VolunteerRecords {
    Table,
    IsDeleted,
}

#[derive(Iden)]
enum ContestRecords {
    Table,
    IsDeleted,
}
