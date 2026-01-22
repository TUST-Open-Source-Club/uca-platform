use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::Email).string().null())
                    .add_column(ColumnDef::new(Users::PasswordHash).string().null())
                    .add_column(
                        ColumnDef::new(Users::AllowPasswordLogin)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(ColumnDef::new(Users::PasswordUpdatedAt).timestamp_with_time_zone().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Invites::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Invites::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Invites::TokenHash).string().not_null())
                    .col(ColumnDef::new(Invites::Email).string().not_null())
                    .col(ColumnDef::new(Invites::Username).string().not_null())
                    .col(ColumnDef::new(Invites::DisplayName).string().not_null())
                    .col(ColumnDef::new(Invites::Role).string().not_null())
                    .col(ColumnDef::new(Invites::ExpiresAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Invites::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Invites::UsedAt).timestamp_with_time_zone().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AuthResets::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AuthResets::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(AuthResets::TokenHash).string().not_null())
                    .col(ColumnDef::new(AuthResets::UserId).uuid().not_null())
                    .col(ColumnDef::new(AuthResets::Purpose).string().not_null())
                    .col(ColumnDef::new(AuthResets::ExpiresAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(AuthResets::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(AuthResets::UsedAt).timestamp_with_time_zone().null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(AuthResets::Table, AuthResets::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PasswordPolicies::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PasswordPolicies::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(PasswordPolicies::MinLength).integer().not_null())
                    .col(ColumnDef::new(PasswordPolicies::RequireUppercase).boolean().not_null())
                    .col(ColumnDef::new(PasswordPolicies::RequireLowercase).boolean().not_null())
                    .col(ColumnDef::new(PasswordPolicies::RequireDigit).boolean().not_null())
                    .col(ColumnDef::new(PasswordPolicies::RequireSymbol).boolean().not_null())
                    .col(ColumnDef::new(PasswordPolicies::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(PasswordPolicies::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PasswordPolicies::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AuthResets::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Invites::Table).to_owned())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::PasswordUpdatedAt)
                    .drop_column(Users::AllowPasswordLogin)
                    .drop_column(Users::PasswordHash)
                    .drop_column(Users::Email)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
    PasswordHash,
    AllowPasswordLogin,
    PasswordUpdatedAt,
}

#[derive(DeriveIden)]
enum Invites {
    Table,
    Id,
    TokenHash,
    Email,
    Username,
    DisplayName,
    Role,
    ExpiresAt,
    CreatedAt,
    UsedAt,
}

#[derive(DeriveIden)]
enum AuthResets {
    Table,
    Id,
    TokenHash,
    UserId,
    Purpose,
    ExpiresAt,
    CreatedAt,
    UsedAt,
}

#[derive(DeriveIden)]
enum PasswordPolicies {
    Table,
    Id,
    MinLength,
    RequireUppercase,
    RequireLowercase,
    RequireDigit,
    RequireSymbol,
    CreatedAt,
    UpdatedAt,
}
