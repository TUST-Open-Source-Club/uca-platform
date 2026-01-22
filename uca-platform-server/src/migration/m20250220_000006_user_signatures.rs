//! 用户电子签名与审核人字段。

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .add_column(ColumnDef::new(ContestRecords::FirstReviewerId).uuid().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .add_column(ColumnDef::new(ContestRecords::FinalReviewerId).uuid().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UserSignatures::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserSignatures::UserId).uuid().not_null().primary_key())
                    .col(ColumnDef::new(UserSignatures::SignaturePath).text().not_null())
                    .col(ColumnDef::new(UserSignatures::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(UserSignatures::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserSignatures::Table, UserSignatures::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserSignatures::Table).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .drop_column(ContestRecords::FinalReviewerId)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .drop_column(ContestRecords::FirstReviewerId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum ContestRecords {
    Table,
    FirstReviewerId,
    FinalReviewerId,
}

#[derive(DeriveIden)]
enum UserSignatures {
    Table,
    UserId,
    SignaturePath,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
