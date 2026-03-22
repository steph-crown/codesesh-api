use sqlx::PgPool;
use uuid::Uuid;

use crate::models::SessionNote;

pub async fn find_by_session_and_user(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
) -> Result<Option<SessionNote>, sqlx::Error> {
  sqlx::query_as::<_, SessionNote>(
    r#"
    SELECT *
    FROM session_notes
    WHERE session_id = $1 AND user_id = $2
    "#,
  )
  .bind(session_id)
  .bind(user_id)
  .fetch_optional(pool)
  .await
}

pub async fn upsert(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
  content: &str,
) -> Result<SessionNote, sqlx::Error> {
  sqlx::query_as::<_, SessionNote>(
    r#"
    INSERT INTO session_notes (session_id, user_id, content)
    VALUES ($1, $2, $3)
    ON CONFLICT (session_id, user_id)
    DO UPDATE SET
      content = EXCLUDED.content,
      updated_at = now()
    RETURNING *
    "#,
  )
  .bind(session_id)
  .bind(user_id)
  .bind(content)
  .fetch_one(pool)
  .await
}
