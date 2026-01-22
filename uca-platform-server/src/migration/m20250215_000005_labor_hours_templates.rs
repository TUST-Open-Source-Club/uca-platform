//! 劳动学时规则与模板配置表。

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(CompetitionLibrary::Table)
                    .add_column(ColumnDef::new(CompetitionLibrary::Year).integer().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(CompetitionLibrary::Table)
                    .add_column(ColumnDef::new(CompetitionLibrary::Category).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .add_column(ColumnDef::new(ContestRecords::ContestLevel).string().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .add_column(ColumnDef::new(ContestRecords::ContestRole).string().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .add_column(ColumnDef::new(ContestRecords::ContestYear).integer().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .add_column(ColumnDef::new(ContestRecords::ContestCategory).string().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .add_column(ColumnDef::new(ContestRecords::AwardDate).timestamp_with_time_zone().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(LaborHourRules::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(LaborHourRules::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(LaborHourRules::BaseHoursA).integer().not_null())
                    .col(ColumnDef::new(LaborHourRules::BaseHoursB).integer().not_null())
                    .col(ColumnDef::new(LaborHourRules::NationalLeaderHours).integer().not_null())
                    .col(ColumnDef::new(LaborHourRules::NationalMemberHours).integer().not_null())
                    .col(ColumnDef::new(LaborHourRules::ProvincialLeaderHours).integer().not_null())
                    .col(ColumnDef::new(LaborHourRules::ProvincialMemberHours).integer().not_null())
                    .col(ColumnDef::new(LaborHourRules::SchoolLeaderHours).integer().not_null())
                    .col(ColumnDef::new(LaborHourRules::SchoolMemberHours).integer().not_null())
                    .col(ColumnDef::new(LaborHourRules::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(LaborHourRules::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ImportTemplates::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ImportTemplates::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(ImportTemplates::TemplateKey).string().not_null())
                    .col(ColumnDef::new(ImportTemplates::Name).string().not_null())
                    .col(ColumnDef::new(ImportTemplates::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(ImportTemplates::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ImportTemplateFields::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ImportTemplateFields::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(ImportTemplateFields::TemplateId).uuid().not_null())
                    .col(ColumnDef::new(ImportTemplateFields::FieldKey).string().not_null())
                    .col(ColumnDef::new(ImportTemplateFields::Label).string().not_null())
                    .col(ColumnDef::new(ImportTemplateFields::ColumnTitle).string().not_null())
                    .col(ColumnDef::new(ImportTemplateFields::Required).boolean().not_null().default(false))
                    .col(ColumnDef::new(ImportTemplateFields::OrderIndex).integer().not_null())
                    .col(ColumnDef::new(ImportTemplateFields::Description).text())
                    .col(ColumnDef::new(ImportTemplateFields::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(ImportTemplateFields::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(ImportTemplateFields::Table, ImportTemplateFields::TemplateId)
                            .to(ImportTemplates::Table, ImportTemplates::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ExportTemplates::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ExportTemplates::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(ExportTemplates::TemplateKey).string().not_null())
                    .col(ColumnDef::new(ExportTemplates::Name).string().not_null())
                    .col(ColumnDef::new(ExportTemplates::LayoutJson).text().not_null())
                    .col(ColumnDef::new(ExportTemplates::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(ExportTemplates::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ExportTemplates::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ImportTemplateFields::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ImportTemplates::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(LaborHourRules::Table).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .drop_column(ContestRecords::AwardDate)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .drop_column(ContestRecords::ContestCategory)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .drop_column(ContestRecords::ContestYear)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .drop_column(ContestRecords::ContestRole)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ContestRecords::Table)
                    .drop_column(ContestRecords::ContestLevel)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(CompetitionLibrary::Table)
                    .drop_column(CompetitionLibrary::Category)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(CompetitionLibrary::Table)
                    .drop_column(CompetitionLibrary::Year)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum CompetitionLibrary {
    Table,
    Year,
    Category,
}

#[derive(Iden)]
enum ContestRecords {
    Table,
    ContestLevel,
    ContestRole,
    ContestYear,
    ContestCategory,
    AwardDate,
}

#[derive(Iden)]
enum LaborHourRules {
    Table,
    Id,
    BaseHoursA,
    BaseHoursB,
    NationalLeaderHours,
    NationalMemberHours,
    ProvincialLeaderHours,
    ProvincialMemberHours,
    SchoolLeaderHours,
    SchoolMemberHours,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum ImportTemplates {
    Table,
    Id,
    TemplateKey,
    Name,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum ImportTemplateFields {
    Table,
    Id,
    TemplateId,
    FieldKey,
    Label,
    ColumnTitle,
    Required,
    OrderIndex,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum ExportTemplates {
    Table,
    Id,
    TemplateKey,
    Name,
    LayoutJson,
    CreatedAt,
    UpdatedAt,
}
