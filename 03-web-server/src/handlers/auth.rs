use crate::{
    dto::auth::{
        AccessTokenResponse, LoginRequest, LogoutRequest, RefreshRequest, RegisterRequest,
        RegisterResponse, TokenResponse,
    },
    error::AppError,
    middleware::Claims,
    services::auth as auth_service,
    state::AppState,
};
use axum::{Extension, Json, extract::State, http::StatusCode};

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    let response = auth_service::register(&state.db, &body.username, &body.password).await?;
    Ok(Json(response))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let tokens = auth_service::login(&state.db, &body.username, &body.password).await?;
    Ok(Json(tokens))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<AccessTokenResponse>, AppError> {
    let token = auth_service::refresh(&state.db, &body.refresh_token).await?;
    Ok(Json(token))
}

pub async fn logout(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(body): Json<LogoutRequest>,
) -> Result<StatusCode, AppError> {
    auth_service::logout(&state.db, &body.refresh_token).await?;
    Ok(StatusCode::NO_CONTENT)
}
