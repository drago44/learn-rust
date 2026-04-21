use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260421_add_auth"
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    Username,
    PasswordHash,
    CreatedAt,
}

#[derive(Iden)]
enum RefreshTokens {
    Table,
    Id,
    UserId,
    ExpiresAt,
    CreatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).string().not_null().primary_key())
                    .col(
                        ColumnDef::new(Users::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(ColumnDef::new(Users::CreatedAt).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RefreshTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RefreshTokens::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RefreshTokens::UserId).string().not_null())
                    .col(ColumnDef::new(RefreshTokens::ExpiresAt).string().not_null())
                    .col(ColumnDef::new(RefreshTokens::CreatedAt).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(RefreshTokens::Table, RefreshTokens::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(RefreshTokens::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).if_exists().to_owned())
            .await
    }
}
