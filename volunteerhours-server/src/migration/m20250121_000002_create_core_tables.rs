//! 创建学生、记录、附件与竞赛基础表。

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Students::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Students::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Students::StudentNo).string().not_null())
                    .col(ColumnDef::new(Students::Name).string().not_null())
                    .col(ColumnDef::new(Students::Gender).string().not_null())
                    .col(ColumnDef::new(Students::Department).string().not_null())
                    .col(ColumnDef::new(Students::Major).string().not_null())
                    .col(ColumnDef::new(Students::ClassName).string().not_null())
                    .col(ColumnDef::new(Students::Phone).string().not_null())
                    .col(ColumnDef::new(Students::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Students::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(VolunteerRecords::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(VolunteerRecords::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(VolunteerRecords::StudentId).uuid().not_null())
                    .col(ColumnDef::new(VolunteerRecords::Title).string().not_null())
                    .col(ColumnDef::new(VolunteerRecords::Description).text().not_null())
                    .col(ColumnDef::new(VolunteerRecords::SelfHours).integer().not_null())
                    .col(ColumnDef::new(VolunteerRecords::FirstReviewHours).integer())
                    .col(ColumnDef::new(VolunteerRecords::FinalReviewHours).integer())
                    .col(ColumnDef::new(VolunteerRecords::Status).string().not_null())
                    .col(ColumnDef::new(VolunteerRecords::RejectionReason).text())
                    .col(ColumnDef::new(VolunteerRecords::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(VolunteerRecords::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(VolunteerRecords::Table, VolunteerRecords::StudentId)
                            .to(Students::Table, Students::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ContestRecords::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ContestRecords::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(ContestRecords::StudentId).uuid().not_null())
                    .col(ColumnDef::new(ContestRecords::ContestName).string().not_null())
                    .col(ColumnDef::new(ContestRecords::AwardLevel).string().not_null())
                    .col(ColumnDef::new(ContestRecords::SelfHours).integer().not_null())
                    .col(ColumnDef::new(ContestRecords::FirstReviewHours).integer())
                    .col(ColumnDef::new(ContestRecords::FinalReviewHours).integer())
                    .col(ColumnDef::new(ContestRecords::Status).string().not_null())
                    .col(ColumnDef::new(ContestRecords::RejectionReason).text())
                    .col(ColumnDef::new(ContestRecords::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(ContestRecords::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(ContestRecords::Table, ContestRecords::StudentId)
                            .to(Students::Table, Students::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Attachments::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Attachments::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Attachments::StudentId).uuid().not_null())
                    .col(ColumnDef::new(Attachments::RecordType).string().not_null())
                    .col(ColumnDef::new(Attachments::RecordId).uuid().not_null())
                    .col(ColumnDef::new(Attachments::OriginalName).string().not_null())
                    .col(ColumnDef::new(Attachments::StoredName).string().not_null())
                    .col(ColumnDef::new(Attachments::MimeType).string().not_null())
                    .col(ColumnDef::new(Attachments::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Attachments::Table, Attachments::StudentId)
                            .to(Students::Table, Students::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(CompetitionLibrary::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(CompetitionLibrary::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(CompetitionLibrary::Name).string().not_null())
                    .col(ColumnDef::new(CompetitionLibrary::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(CompetitionLibrary::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ReviewSignatures::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ReviewSignatures::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(ReviewSignatures::RecordType).string().not_null())
                    .col(ColumnDef::new(ReviewSignatures::RecordId).uuid().not_null())
                    .col(ColumnDef::new(ReviewSignatures::ReviewerUserId).uuid().not_null())
                    .col(ColumnDef::new(ReviewSignatures::Stage).string().not_null())
                    .col(ColumnDef::new(ReviewSignatures::SignaturePath).string().not_null())
                    .col(ColumnDef::new(ReviewSignatures::CreatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(FormFields::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(FormFields::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(FormFields::FormType).string().not_null())
                    .col(ColumnDef::new(FormFields::FieldKey).string().not_null())
                    .col(ColumnDef::new(FormFields::Label).string().not_null())
                    .col(ColumnDef::new(FormFields::FieldType).string().not_null())
                    .col(ColumnDef::new(FormFields::Required).boolean().not_null().default(false))
                    .col(ColumnDef::new(FormFields::OrderIndex).integer().not_null())
                    .col(ColumnDef::new(FormFields::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(FormFields::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(FormFieldValues::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(FormFieldValues::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(FormFieldValues::RecordType).string().not_null())
                    .col(ColumnDef::new(FormFieldValues::RecordId).uuid().not_null())
                    .col(ColumnDef::new(FormFieldValues::FieldKey).string().not_null())
                    .col(ColumnDef::new(FormFieldValues::Value).text().not_null())
                    .col(ColumnDef::new(FormFieldValues::CreatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FormFieldValues::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(FormFields::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ReviewSignatures::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CompetitionLibrary::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Attachments::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ContestRecords::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(VolunteerRecords::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Students::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Students {
    Table,
    Id,
    StudentNo,
    Name,
    Gender,
    Department,
    Major,
    ClassName,
    Phone,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum VolunteerRecords {
    Table,
    Id,
    StudentId,
    Title,
    Description,
    SelfHours,
    FirstReviewHours,
    FinalReviewHours,
    Status,
    RejectionReason,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum ContestRecords {
    Table,
    Id,
    StudentId,
    ContestName,
    AwardLevel,
    SelfHours,
    FirstReviewHours,
    FinalReviewHours,
    Status,
    RejectionReason,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Attachments {
    Table,
    Id,
    StudentId,
    RecordType,
    RecordId,
    OriginalName,
    StoredName,
    MimeType,
    CreatedAt,
}

#[derive(Iden)]
enum CompetitionLibrary {
    Table,
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum ReviewSignatures {
    Table,
    Id,
    RecordType,
    RecordId,
    ReviewerUserId,
    Stage,
    SignaturePath,
    CreatedAt,
}

#[derive(Iden)]
enum FormFields {
    Table,
    Id,
    FormType,
    FieldKey,
    Label,
    FieldType,
    Required,
    OrderIndex,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum FormFieldValues {
    Table,
    Id,
    RecordType,
    RecordId,
    FieldKey,
    Value,
    CreatedAt,
}
