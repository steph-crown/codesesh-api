use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Session, SessionLanguage};

/// Meet-style code: three groups of three random English letters, hyphen-separated (`abc-def-ghi`).
pub fn new_short_id() -> String {
  let mut rng = rand::thread_rng();
  let mut chars: Vec<char> = (0..9).map(|_| rng.gen_range(b'a'..=b'z') as char).collect();
  chars.insert(3, '-');
  chars.insert(7, '-');
  chars.into_iter().collect()
}

fn is_unique_violation(e: &sqlx::Error) -> bool {
  e.as_database_error()
    .and_then(|d| d.code().map(|c| c.as_ref() == "23505"))
    .unwrap_or(false)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Session>, sqlx::Error> {
  sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = $1")
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_short_id(
  pool: &PgPool,
  short_id: &str,
) -> Result<Option<Session>, sqlx::Error> {
  sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE short_id = $1")
    .bind(short_id)
    .fetch_optional(pool)
    .await
}

pub async fn insert(
  pool: &PgPool,
  host_id: Uuid,
  name: &str,
  language: SessionLanguage,
) -> Result<Session, sqlx::Error> {
  for _ in 0..16 {
    let short_id = new_short_id();
    let res = sqlx::query_as::<_, Session>(
      r#"
      INSERT INTO sessions (short_id, host_id, name, language)
      VALUES ($1, $2, $3, $4)
      RETURNING *
      "#,
    )
    .bind(&short_id)
    .bind(host_id)
    .bind(name)
    .bind(&language)
    .fetch_one(pool)
    .await;

    match res {
      Ok(session) => return Ok(session),
      Err(e) if is_unique_violation(&e) => continue,
      Err(e) => return Err(e),
    }
  }

  Err(sqlx::Error::Protocol(
    "could not allocate unique short_id".to_string(),
  ))
}

/// Sessions the user hosts or participates in, with optional filters.
pub async fn count_accessible(
  pool: &PgPool,
  user_id: Uuid,
  search: Option<&str>,
  created_only: bool,
  shared_only: bool,
) -> Result<i64, sqlx::Error> {
  let search_pattern = search.map(|s| format!("%{s}%"));

  if created_only && !shared_only {
    sqlx::query_scalar::<_, i64>(
      r#"
      SELECT COUNT(*)::bigint
      FROM sessions s
      WHERE s.host_id = $1
        AND ($2::text IS NULL OR s.name ILIKE $2)
      "#,
    )
    .bind(user_id)
    .bind(search_pattern.as_deref())
    .fetch_one(pool)
    .await
  } else if shared_only && !created_only {
    sqlx::query_scalar::<_, i64>(
      r#"
      SELECT COUNT(*)::bigint
      FROM sessions s
      INNER JOIN session_participants sp ON sp.session_id = s.id AND sp.user_id = $1
      WHERE s.host_id <> $1
        AND ($2::text IS NULL OR s.name ILIKE $2)
      "#,
    )
    .bind(user_id)
    .bind(search_pattern.as_deref())
    .fetch_one(pool)
    .await
  } else {
    sqlx::query_scalar::<_, i64>(
      r#"
      SELECT COUNT(*)::bigint
      FROM sessions s
      WHERE (
        s.host_id = $1
        OR EXISTS (
          SELECT 1 FROM session_participants sp
          WHERE sp.session_id = s.id AND sp.user_id = $1
        )
      )
      AND ($2::text IS NULL OR s.name ILIKE $2)
      "#,
    )
    .bind(user_id)
    .bind(search_pattern.as_deref())
    .fetch_one(pool)
    .await
  }
}

pub async fn list_accessible(
  pool: &PgPool,
  user_id: Uuid,
  search: Option<&str>,
  created_only: bool,
  shared_only: bool,
  page: i64,
  limit: i64,
) -> Result<Vec<Session>, sqlx::Error> {
  let offset = (page - 1) * limit;
  let search_pattern = search.map(|s| format!("%{s}%"));

  if created_only && !shared_only {
    sqlx::query_as::<_, Session>(
      r#"
      SELECT s.*
      FROM sessions s
      WHERE s.host_id = $1
        AND ($2::text IS NULL OR s.name ILIKE $2)
      ORDER BY s.last_activity_at DESC
      LIMIT $3 OFFSET $4
      "#,
    )
    .bind(user_id)
    .bind(search_pattern.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
  } else if shared_only && !created_only {
    sqlx::query_as::<_, Session>(
      r#"
      SELECT s.*
      FROM sessions s
      INNER JOIN session_participants sp ON sp.session_id = s.id AND sp.user_id = $1
      WHERE s.host_id <> $1
        AND ($2::text IS NULL OR s.name ILIKE $2)
      ORDER BY s.last_activity_at DESC
      LIMIT $3 OFFSET $4
      "#,
    )
    .bind(user_id)
    .bind(search_pattern.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
  } else {
    sqlx::query_as::<_, Session>(
      r#"
      SELECT s.*
      FROM sessions s
      WHERE (
        s.host_id = $1
        OR EXISTS (
          SELECT 1 FROM session_participants sp
          WHERE sp.session_id = s.id AND sp.user_id = $1
        )
      )
      AND ($2::text IS NULL OR s.name ILIKE $2)
      ORDER BY s.last_activity_at DESC
      LIMIT $3 OFFSET $4
      "#,
    )
    .bind(user_id)
    .bind(search_pattern.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
  }
}

pub async fn update_name(
  pool: &PgPool,
  session_id: Uuid,
  name: &str,
) -> Result<Session, sqlx::Error> {
  sqlx::query_as::<_, Session>(
    r#"
    UPDATE sessions
    SET name = $2, updated_at = now(), last_activity_at = now()
    WHERE id = $1
    RETURNING *
    "#,
  )
  .bind(session_id)
  .bind(name)
  .fetch_one(pool)
  .await
}

pub async fn update_visibility(
  pool: &PgPool,
  session_id: Uuid,
  visibility: crate::models::SessionVisibility,
) -> Result<Session, sqlx::Error> {
  sqlx::query_as::<_, Session>(
    r#"
    UPDATE sessions
    SET visibility = $2, updated_at = now(), last_activity_at = now()
    WHERE id = $1
    RETURNING *
    "#,
  )
  .bind(session_id)
  .bind(visibility)
  .fetch_one(pool)
  .await
}

pub async fn set_ended(pool: &PgPool, session_id: Uuid) -> Result<Session, sqlx::Error> {
  sqlx::query_as::<_, Session>(
    r#"
    UPDATE sessions
    SET status = 'ended', updated_at = now(), last_activity_at = now()
    WHERE id = $1
    RETURNING *
    "#,
  )
  .bind(session_id)
  .fetch_one(pool)
  .await
}
