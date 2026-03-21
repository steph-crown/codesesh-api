use sqlx::PgPool;
use uuid::Uuid;

use crate::models::User;

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
  sqlx::query_as::<_, User>("SELECT id, display_name, created_at FROM users WHERE id = $1")
    .bind(id)
    .fetch_optional(pool)
    .await
}
