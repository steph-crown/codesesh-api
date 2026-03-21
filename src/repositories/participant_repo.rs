use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::SessionParticipant;

#[derive(sqlx::FromRow)]
struct ParticipantWithNameRow {
  id: Uuid,
  session_id: Uuid,
  user_id: Uuid,
  joined_at: OffsetDateTime,
  left_at: Option<OffsetDateTime>,
  display_name: String,
  color: String,
}

pub async fn list_with_display_names(
  pool: &PgPool,
  session_id: Uuid,
) -> Result<Vec<(SessionParticipant, String, String)>, sqlx::Error> {
  let rows: Vec<ParticipantWithNameRow> = sqlx::query_as(
    r#"
    SELECT
      sp.id,
      sp.session_id,
      sp.user_id,
      sp.joined_at,
      sp.left_at,
      u.display_name,
      u.color
    FROM session_participants sp
    INNER JOIN users u ON u.id = sp.user_id
    WHERE sp.session_id = $1
    ORDER BY sp.joined_at ASC
    "#,
  )
  .bind(session_id)
  .fetch_all(pool)
  .await?;

  Ok(rows
    .into_iter()
    .map(|r| {
      (
        SessionParticipant {
          id: r.id,
          session_id: r.session_id,
          user_id: r.user_id,
          joined_at: r.joined_at,
          left_at: r.left_at,
        },
        r.display_name,
        r.color,
      )
    })
    .collect())
}

/// Whether the user has an active (not left) participant row for this session.
pub async fn is_active_participant(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
) -> Result<bool, sqlx::Error> {
  let exists: bool = sqlx::query_scalar(
    r#"
    SELECT EXISTS(
      SELECT 1
      FROM session_participants
      WHERE session_id = $1 AND user_id = $2 AND left_at IS NULL
    )
    "#,
  )
  .bind(session_id)
  .bind(user_id)
  .fetch_one(pool)
  .await?;
  Ok(exists)
}

/// Idempotent join: reactivate row if user already participated.
pub async fn upsert_active(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
) -> Result<SessionParticipant, sqlx::Error> {
  sqlx::query_as::<_, SessionParticipant>(
    r#"
    INSERT INTO session_participants (session_id, user_id)
    VALUES ($1, $2)
    ON CONFLICT (session_id, user_id)
    DO UPDATE SET left_at = NULL
    RETURNING id, session_id, user_id, joined_at, left_at
    "#,
  )
  .bind(session_id)
  .bind(user_id)
  .fetch_one(pool)
  .await
}
