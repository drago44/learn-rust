use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub id: String,
    pub username: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[derive(Serialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
}
