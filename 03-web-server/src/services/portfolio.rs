use crate::{
    dto::portfolio::{AssetResponse, PortfolioResponse},
    error::AppError,
    repositories::portfolio as portfolio_repo,
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

pub async fn create_portfolio(
    db: &DatabaseConnection,
    user_id: &str,
    name: &str,
) -> Result<PortfolioResponse, AppError> {
    if portfolio_repo::find_by_user_id(db, user_id)
        .await
        .map_err(AppError::Internal)?
        .is_some()
    {
        return Err(AppError::Conflict("portfolio already exists".to_string()));
    }

    let id = Uuid::new_v4().to_string();
    let portfolio = portfolio_repo::create(db, &id, user_id, name)
        .await
        .map_err(AppError::Internal)?;

    Ok(PortfolioResponse {
        id: portfolio.id,
        name: portfolio.name,
        assets: vec![],
    })
}

pub async fn get_portfolio(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<PortfolioResponse, AppError> {
    let portfolio = portfolio_repo::find_by_user_id(db, user_id)
        .await
        .map_err(AppError::Internal)?
        .ok_or_else(|| AppError::NotFound("portfolio not found".to_string()))?;

    let assets = portfolio_repo::find_assets(db, &portfolio.id)
        .await
        .map_err(AppError::Internal)?
        .into_iter()
        .map(|a| AssetResponse {
            id: a.id,
            symbol: a.symbol,
            amount: a.amount,
        })
        .collect();

    Ok(PortfolioResponse {
        id: portfolio.id,
        name: portfolio.name,
        assets,
    })
}

pub async fn add_asset(
    db: &DatabaseConnection,
    user_id: &str,
    symbol: &str,
    amount: f64,
) -> Result<AssetResponse, AppError> {
    let portfolio = portfolio_repo::find_by_user_id(db, user_id)
        .await
        .map_err(AppError::Internal)?
        .ok_or_else(|| AppError::NotFound("portfolio not found".to_string()))?;

    if portfolio_repo::find_asset_by_symbol(db, &portfolio.id, symbol)
        .await
        .map_err(AppError::Internal)?
        .is_some()
    {
        return Err(AppError::Conflict(format!(
            "asset '{}' already exists in portfolio",
            symbol
        )));
    }

    let id = Uuid::new_v4().to_string();
    let asset = portfolio_repo::add_asset(db, &id, &portfolio.id, symbol, amount)
        .await
        .map_err(AppError::Internal)?;

    Ok(AssetResponse {
        id: asset.id,
        symbol: asset.symbol,
        amount: asset.amount,
    })
}

pub async fn delete_asset(
    db: &DatabaseConnection,
    user_id: &str,
    symbol: &str,
) -> Result<(), AppError> {
    let portfolio = portfolio_repo::find_by_user_id(db, user_id)
        .await
        .map_err(AppError::Internal)?
        .ok_or_else(|| AppError::NotFound("portfolio not found".to_string()))?;

    let deleted = portfolio_repo::delete_asset(db, &portfolio.id, symbol)
        .await
        .map_err(AppError::Internal)?;

    if !deleted {
        return Err(AppError::NotFound(format!("asset '{}' not found", symbol)));
    }

    Ok(())
}
