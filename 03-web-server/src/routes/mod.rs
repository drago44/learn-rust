use crate::{handlers, middleware::jwt_auth, state::AppState};
use axum::{
    Router, middleware,
    routing::{delete, get, post},
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};

pub fn routes(db: DatabaseConnection) -> Router {
    // суворий — brute force захист на auth
    let auth_governor = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .finish()
            .unwrap(),
    );

    // м'який — DoS захист на всі маршрути
    let global_governor = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(100)
            .finish()
            .unwrap(),
    );

    let auth_public = Router::new()
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh))
        .layer(GovernorLayer::new(auth_governor));

    let auth_protected = Router::new()
        .route("/auth/logout", post(handlers::auth::logout))
        .layer(middleware::from_fn(jwt_auth));

    let portfolio = Router::new()
        .route("/portfolio", post(handlers::portfolio::create_portfolio))
        .route("/portfolio", get(handlers::portfolio::get_portfolio))
        .route("/portfolio/asset", post(handlers::portfolio::add_asset))
        .route(
            "/portfolio/asset/{symbol}",
            delete(handlers::portfolio::delete_asset),
        )
        .layer(middleware::from_fn(jwt_auth));

    let public = Router::new()
        .route("/coins", get(handlers::coins::get_coins))
        .route("/prices/{symbol}", get(handlers::prices::get_price))
        .route("/health", get(handlers::health::health_handler));

    Router::new()
        .nest(
            "/api/v1",
            auth_public
                .merge(auth_protected)
                .merge(portfolio)
                .merge(public),
        )
        .layer(GovernorLayer::new(global_governor))
        .with_state(AppState { db })
}
