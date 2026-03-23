use sqlx::PgPool;
use uuid::Uuid;

use crate::models::SessionLanguage;

pub async fn insert(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
  language: SessionLanguage,
) -> Result<Uuid, sqlx::Error> {
  let row = sqlx::query_scalar::<_, Uuid>(
    r#"
    INSERT INTO code_runs (session_id, user_id, language)
    VALUES ($1, $2, $3)
    RETURNING id
    "#,
  )
  .bind(session_id)
  .bind(user_id)
  .bind(language)
  .fetch_one(pool)
  .await?;

  Ok(row)
}
