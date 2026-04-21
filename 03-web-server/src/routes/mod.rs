use crate::{handlers, middleware::jwt_auth, state::AppState};
use axum::{
    Router, middleware,
    routing::{get, post},
};
use sea_orm::DatabaseConnection;

pub fn routes(db: DatabaseConnection) -> Router {
    let auth_public = Router::new()
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh));

    let auth_protected = Router::new()
        .route("/auth/logout", post(handlers::auth::logout))
        .layer(middleware::from_fn(jwt_auth));

    let public = Router::new()
        .route("/coins", get(handlers::coins::get_coins))
        .route("/prices/{symbol}", get(handlers::prices::get_price))
        .route("/health", get(handlers::health::health_handler));

    Router::new()
        .nest("/api/v1", auth_public.merge(auth_protected).merge(public))
        .with_state(AppState { db })
}
