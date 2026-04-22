use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Portfolios::Table)
                    .if_not_exists()
                    .col(string(Portfolios::Id).primary_key())
                    .col(string(Portfolios::UserId).not_null())
                    .col(string(Portfolios::Name).not_null())
                    .col(timestamp(Portfolios::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Portfolios::Table, Portfolios::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PortfolioAssets::Table)
                    .if_not_exists()
                    .col(string(PortfolioAssets::Id).primary_key())
                    .col(string(PortfolioAssets::PortfolioId).not_null())
                    .col(string(PortfolioAssets::Symbol).not_null())
                    .col(double(PortfolioAssets::Amount).not_null())
                    .col(timestamp(PortfolioAssets::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(PortfolioAssets::Table, PortfolioAssets::PortfolioId)
                            .to(Portfolios::Table, Portfolios::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PortfolioAssets::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Portfolios::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Portfolios {
    Table,
    Id,
    UserId,
    Name,
    CreatedAt,
}

#[derive(DeriveIden)]
enum PortfolioAssets {
    Table,
    Id,
    PortfolioId,
    Symbol,
    Amount,
    CreatedAt,
}

// Посилання на існуючу таблицю для FK
#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
