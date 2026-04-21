use crate::{
    dto::auth::{AccessTokenResponse, RegisterResponse, TokenResponse},
    error::AppError,
    middleware::Claims,
    repositories::{auth as auth_repo, user as user_repo},
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes())
        .map_err(|_| AppError::Internal(anyhow::anyhow!("salt error")))?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AppError::Internal(anyhow::anyhow!("password hash error")))
}

fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .map(|h| {
            Argon2::default()
                .verify_password(password.as_bytes(), &h)
                .is_ok()
        })
        .unwrap_or(false)
}

fn generate_access_token(user_id: &str) -> Result<String, AppError> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let exp = chrono::Utc::now().timestamp() as usize + 15 * 60;
    let claims = Claims {
        sub: user_id.to_string(),
        exp,
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AppError::Internal(anyhow::anyhow!("jwt encode error")))
}

pub async fn register(
    db: &DatabaseConnection,
    username: &str,
    password: &str,
) -> Result<RegisterResponse, AppError> {
    if user_repo::find_by_username(db, username)
        .await
        .map_err(AppError::Internal)?
        .is_some()
    {
        return Err(AppError::Conflict(format!(
            "username '{}' already taken",
            username
        )));
    }

    let id = Uuid::new_v4().to_string();
    let hash = hash_password(password)?;
    user_repo::create(db, &id, username, &hash)
        .await
        .map_err(AppError::Internal)?;

    Ok(RegisterResponse {
        id,
        username: username.to_string(),
    })
}

pub async fn login(
    db: &DatabaseConnection,
    username: &str,
    password: &str,
) -> Result<TokenResponse, AppError> {
    let user = user_repo::find_by_username(db, username)
        .await
        .map_err(AppError::Internal)?
        .ok_or_else(|| AppError::Unauthorized("invalid credentials".to_string()))?;

    if !verify_password(password, &user.password_hash) {
        return Err(AppError::Unauthorized("invalid credentials".to_string()));
    }

    let access_token = generate_access_token(&user.id)?;
    let refresh_token = Uuid::new_v4().to_string();
    let expires_at = (chrono::Utc::now() + chrono::Duration::days(7)).to_rfc3339();

    auth_repo::create_refresh_token(db, &refresh_token, &user.id, &expires_at)
        .await
        .map_err(AppError::Internal)?;

    Ok(TokenResponse {
        access_token,
        refresh_token,
        expires_in: 900,
    })
}

pub async fn refresh(
    db: &DatabaseConnection,
    refresh_token: &str,
) -> Result<AccessTokenResponse, AppError> {
    let (user_id, expires_at) = auth_repo::find_refresh_token(db, refresh_token)
        .await
        .map_err(AppError::Internal)?
        .ok_or_else(|| AppError::Unauthorized("invalid refresh token".to_string()))?;

    let expires = chrono::DateTime::parse_from_rfc3339(&expires_at)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("bad expires_at in db")))?;

    if expires < chrono::Utc::now() {
        auth_repo::delete_refresh_token(db, refresh_token)
            .await
            .map_err(AppError::Internal)?;
        return Err(AppError::Unauthorized("refresh token expired".to_string()));
    }

    let access_token = generate_access_token(&user_id)?;
    Ok(AccessTokenResponse {
        access_token,
        expires_in: 900,
    })
}

pub async fn logout(db: &DatabaseConnection, refresh_token: &str) -> Result<(), AppError> {
    auth_repo::delete_refresh_token(db, refresh_token)
        .await
        .map_err(AppError::Internal)?;
    Ok(())
}
