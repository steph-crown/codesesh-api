use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct MessageWithAuthor {
  pub id: Uuid,
  pub session_id: Uuid,
  pub user_id: Uuid,
  pub content: String,
  pub created_at: OffsetDateTime,
  pub display_name: String,
  pub color: String,
}

/// Chat history: newest-first page, returned in chronological order (oldest → newest).
pub async fn list_history(
  pool: &PgPool,
  session_id: Uuid,
  limit: i64,
  before: Option<Uuid>,
) -> Result<(Vec<MessageWithAuthor>, bool), sqlx::Error> {
  let fetch = limit + 1;
  let mut rows: Vec<MessageWithAuthor> = sqlx::query_as(
    r#"
    SELECT
      m.id,
      m.session_id,
      m.user_id,
      m.content,
      m.created_at,
      u.display_name,
      u.color
    FROM chat_messages m
    INNER JOIN users u ON u.id = m.user_id
    WHERE m.session_id = $1
      AND (
        $3::uuid IS NULL
        OR m.created_at < (
          SELECT cm.created_at
          FROM chat_messages cm
          WHERE cm.id = $3 AND cm.session_id = $1
        )
      )
    ORDER BY m.created_at DESC
    LIMIT $2
    "#,
  )
  .bind(session_id)
  .bind(fetch)
  .bind(before)
  .fetch_all(pool)
  .await?;

  let has_more = rows.len() as i64 > limit;
  if has_more {
    rows.pop();
  }
  rows.reverse();
  Ok((rows, has_more))
}

/// Insert a chat message and return the same row shape as [`list_history`].
pub async fn insert(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
  content: &str,
) -> Result<MessageWithAuthor, sqlx::Error> {
  sqlx::query_as::<_, MessageWithAuthor>(
    r#"
    WITH ins AS (
      INSERT INTO chat_messages (session_id, user_id, content)
      VALUES ($1, $2, $3)
      RETURNING id, session_id, user_id, content, created_at
    )
    SELECT
      ins.id,
      ins.session_id,
      ins.user_id,
      ins.content,
      ins.created_at,
      u.display_name,
      u.color
    FROM ins
    INNER JOIN users u ON u.id = ins.user_id
    "#,
  )
  .bind(session_id)
  .bind(user_id)
  .bind(content)
  .fetch_one(pool)
  .await
}
