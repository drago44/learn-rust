use crate::models::refresh_token::{self, Entity as RefreshToken};
use anyhow::Result;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

pub async fn create_refresh_token(
    db: &DatabaseConnection,
    id: &str,
    user_id: &str,
    expires_at: &str,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    refresh_token::ActiveModel {
        id: Set(id.to_string()),
        user_id: Set(user_id.to_string()),
        expires_at: Set(expires_at.to_string()),
        created_at: Set(now),
    }
    .insert(db)
    .await?;
    Ok(())
}

pub async fn find_refresh_token(
    db: &DatabaseConnection,
    id: &str,
) -> Result<Option<(String, String)>> {
    let token = RefreshToken::find_by_id(id.to_string())
        .one(db)
        .await?
        .map(|t| (t.user_id, t.expires_at));
    Ok(token)
}

pub async fn delete_refresh_token(db: &DatabaseConnection, id: &str) -> Result<()> {
    RefreshToken::delete_by_id(id.to_string()).exec(db).await?;
    Ok(())
}
