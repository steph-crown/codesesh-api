use sqlx::PgPool;
use uuid::Uuid;

use crate::models::User;

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
  sqlx::query_as::<_, User>(
    "SELECT id, display_name, color, created_at FROM users WHERE id = $1",
  )
  .bind(id)
  .fetch_optional(pool)
  .await
}

pub async fn insert(pool: &PgPool, display_name: &str, color: &str) -> Result<User, sqlx::Error> {
  sqlx::query_as::<_, User>(
    r#"
    INSERT INTO users (display_name, color)
    VALUES ($1, $2)
    RETURNING id, display_name, color, created_at
    "#,
  )
  .bind(display_name)
  .bind(color)
  .fetch_one(pool)
  .await
}
