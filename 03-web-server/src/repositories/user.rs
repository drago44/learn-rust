use crate::models::user::{self, Entity as User};
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub async fn create(db: &DatabaseConnection, id: &str, username: &str, hash: &str) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    user::ActiveModel {
        id: Set(id.to_string()),
        username: Set(username.to_string()),
        password_hash: Set(hash.to_string()),
        created_at: Set(now),
    }
    .insert(db)
    .await?;
    Ok(())
}

pub async fn find_by_username(
    db: &DatabaseConnection,
    username: &str,
) -> Result<Option<user::Model>> {
    let user = User::find()
        .filter(user::Column::Username.eq(username))
        .one(db)
        .await?;
    Ok(user)
}
