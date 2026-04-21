use axum::{
    Json,
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub async fn jwt_auth(mut req: Request, next: Next) -> Response {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let Some(token) = token else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "missing authorization header" })),
        )
            .into_response();
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

    let claims = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(data) => data.claims,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "invalid token" })),
            )
                .into_response();
        }
    };

    req.extensions_mut().insert(claims);
    next.run(req).await
}
