use crate::models::{
    portfolio::{self, Entity as Portfolio},
    portfolio_asset::{self, Entity as PortfolioAsset},
};
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub async fn find_by_user_id(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Option<portfolio::Model>> {
    Ok(Portfolio::find()
        .filter(portfolio::Column::UserId.eq(user_id))
        .one(db)
        .await?)
}

pub async fn create(
    db: &DatabaseConnection,
    id: &str,
    user_id: &str,
    name: &str,
) -> Result<portfolio::Model> {
    let now = chrono::Utc::now().to_rfc3339();
    Ok(portfolio::ActiveModel {
        id: Set(id.to_string()),
        user_id: Set(user_id.to_string()),
        name: Set(name.to_string()),
        created_at: Set(now),
    }
    .insert(db)
    .await?)
}

pub async fn find_assets(
    db: &DatabaseConnection,
    portfolio_id: &str,
) -> Result<Vec<portfolio_asset::Model>> {
    Ok(PortfolioAsset::find()
        .filter(portfolio_asset::Column::PortfolioId.eq(portfolio_id))
        .all(db)
        .await?)
}

pub async fn find_asset_by_symbol(
    db: &DatabaseConnection,
    portfolio_id: &str,
    symbol: &str,
) -> Result<Option<portfolio_asset::Model>> {
    Ok(PortfolioAsset::find()
        .filter(portfolio_asset::Column::PortfolioId.eq(portfolio_id))
        .filter(portfolio_asset::Column::Symbol.eq(symbol))
        .one(db)
        .await?)
}

pub async fn add_asset(
    db: &DatabaseConnection,
    id: &str,
    portfolio_id: &str,
    symbol: &str,
    amount: f64,
) -> Result<portfolio_asset::Model> {
    let now = chrono::Utc::now().to_rfc3339();
    Ok(portfolio_asset::ActiveModel {
        id: Set(id.to_string()),
        portfolio_id: Set(portfolio_id.to_string()),
        symbol: Set(symbol.to_string()),
        amount: Set(amount),
        created_at: Set(now),
    }
    .insert(db)
    .await?)
}

pub async fn delete_asset(
    db: &DatabaseConnection,
    portfolio_id: &str,
    symbol: &str,
) -> Result<bool> {
    let result = PortfolioAsset::delete_many()
        .filter(portfolio_asset::Column::PortfolioId.eq(portfolio_id))
        .filter(portfolio_asset::Column::Symbol.eq(symbol))
        .exec(db)
        .await?;
    Ok(result.rows_affected > 0)
}
