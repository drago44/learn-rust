use crate::{
    dto::portfolio::{AddAssetRequest, AssetResponse, CreatePortfolioRequest, PortfolioResponse},
    error::AppError,
    middleware::Claims,
    services::portfolio as portfolio_service,
    state::AppState,
};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};

pub async fn create_portfolio(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<CreatePortfolioRequest>,
) -> Result<Json<PortfolioResponse>, AppError> {
    let portfolio = portfolio_service::create_portfolio(&state.db, &claims.sub, &body.name).await?;
    Ok(Json(portfolio))
}

pub async fn get_portfolio(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<PortfolioResponse>, AppError> {
    let portfolio = portfolio_service::get_portfolio(&state.db, &claims.sub).await?;
    Ok(Json(portfolio))
}

pub async fn add_asset(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<AddAssetRequest>,
) -> Result<Json<AssetResponse>, AppError> {
    let asset =
        portfolio_service::add_asset(&state.db, &claims.sub, &body.symbol, body.amount).await?;
    Ok(Json(asset))
}

pub async fn delete_asset(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(symbol): Path<String>,
) -> Result<StatusCode, AppError> {
    portfolio_service::delete_asset(&state.db, &claims.sub, &symbol).await?;
    Ok(StatusCode::NO_CONTENT)
}
