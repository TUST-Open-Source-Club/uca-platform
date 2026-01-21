//! 创建认证相关表。

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Users::Username).string().not_null())
                    .col(ColumnDef::new(Users::DisplayName).string().not_null())
                    .col(ColumnDef::new(Users::Role).string().not_null())
                    .col(ColumnDef::new(Users::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(Users::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Users::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Passkeys::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Passkeys::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Passkeys::UserId).uuid().not_null())
                    .col(ColumnDef::new(Passkeys::CredentialId).string().not_null())
                    .col(ColumnDef::new(Passkeys::PasskeyJson).text().not_null())
                    .col(ColumnDef::new(Passkeys::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Passkeys::LastUsedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Passkeys::Table, Passkeys::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TotpSecrets::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TotpSecrets::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(TotpSecrets::UserId).uuid().not_null())
                    .col(ColumnDef::new(TotpSecrets::SecretEnc).text().not_null())
                    .col(ColumnDef::new(TotpSecrets::Enabled).boolean().not_null().default(false))
                    .col(ColumnDef::new(TotpSecrets::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(TotpSecrets::VerifiedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .from(TotpSecrets::Table, TotpSecrets::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RecoveryCodes::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(RecoveryCodes::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(RecoveryCodes::UserId).uuid().not_null())
                    .col(ColumnDef::new(RecoveryCodes::CodeHash).string().not_null())
                    .col(ColumnDef::new(RecoveryCodes::UsedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(RecoveryCodes::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(RecoveryCodes::Table, RecoveryCodes::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Sessions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Sessions::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Sessions::UserId).uuid().not_null())
                    .col(ColumnDef::new(Sessions::TokenHash).string().not_null())
                    .col(ColumnDef::new(Sessions::ExpiresAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Sessions::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Sessions::LastSeenAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Sessions::Table, Sessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Devices::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Devices::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Devices::UserId).uuid().not_null())
                    .col(ColumnDef::new(Devices::DeviceType).string().not_null())
                    .col(ColumnDef::new(Devices::Label).string().not_null())
                    .col(ColumnDef::new(Devices::CredentialId).string())
                    .col(ColumnDef::new(Devices::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Devices::LastUsedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Devices::Table, Devices::UserId)
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
            .drop_table(Table::drop().table(Devices::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Sessions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RecoveryCodes::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TotpSecrets::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Passkeys::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    Username,
    DisplayName,
    Role,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Passkeys {
    Table,
    Id,
    UserId,
    CredentialId,
    PasskeyJson,
    CreatedAt,
    LastUsedAt,
}

#[derive(Iden)]
enum TotpSecrets {
    Table,
    Id,
    UserId,
    SecretEnc,
    Enabled,
    CreatedAt,
    VerifiedAt,
}

#[derive(Iden)]
enum RecoveryCodes {
    Table,
    Id,
    UserId,
    CodeHash,
    UsedAt,
    CreatedAt,
}

#[derive(Iden)]
enum Sessions {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    CreatedAt,
    LastSeenAt,
}

#[derive(Iden)]
enum Devices {
    Table,
    Id,
    UserId,
    DeviceType,
    Label,
    CredentialId,
    CreatedAt,
    LastUsedAt,
}
