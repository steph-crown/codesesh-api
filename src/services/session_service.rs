use sqlx::PgPool;
use uuid::Uuid;

use crate::{
  dto::{
    CreateSessionRequest, GetSessionsQuery, PaginatedResponse, SessionDetailResponse,
    SessionSummaryResponse, UpdateSessionNameRequest, UpdateSessionVisibilityRequest,
  },
  errors::{RepoError, ServiceError},
  models::{Session, SessionStatus},
  repositories::session_repo,
};

pub fn page_limit(page: Option<i64>, limit: Option<i64>) -> (i64, i64) {
  let page = page.unwrap_or(1).max(1);
  let lim = limit.unwrap_or(20).clamp(1, 100);
  (page, lim)
}

fn list_flags(query: &GetSessionsQuery) -> (bool, bool) {
  let created_only = query.created_by_me == Some(true);
  let shared_only = query.shared_with_me == Some(true);
  (created_only, shared_only)
}

pub async fn create_session(
  pool: &PgPool,
  host_id: Uuid,
  req: CreateSessionRequest,
) -> Result<Session, ServiceError> {
  let name = req.name.trim();
  if name.is_empty() {
    return Err(ServiceError::Validation(
      "name must be between 1 and 255 characters".to_string(),
    ));
  }

  let session = session_repo::insert(pool, host_id, name, req.language)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?;

  tracing::info!("session created");
  Ok(session)
}

pub async fn list_sessions(
  pool: &PgPool,
  user_id: Uuid,
  query: GetSessionsQuery,
) -> Result<PaginatedResponse<SessionSummaryResponse>, ServiceError> {
  let (page, limit) = page_limit(query.page, query.limit);
  let (created_only, shared_only) = list_flags(&query);
  let search = query.search.as_deref().map(str::trim).filter(|s| !s.is_empty());

  let total = session_repo::count_accessible(pool, user_id, search, created_only, shared_only)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?;

  let rows = session_repo::list_accessible(
    pool,
    user_id,
    search,
    created_only,
    shared_only,
    page,
    limit,
  )
  .await
  .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?;

  let data = rows
    .into_iter()
    .map(|s| SessionSummaryResponse::from_session(s, user_id))
    .collect();

  Ok(PaginatedResponse::new(data, total, page, limit))
}

pub async fn get_session(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
) -> Result<SessionDetailResponse, ServiceError> {
  let session = session_repo::find_by_id(pool, session_id)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?
    .ok_or(ServiceError::Repo(RepoError::NotFound))?;

  if !session.is_readable_by(user_id) {
    return Err(ServiceError::SessionPrivate);
  }

  Ok(SessionDetailResponse::from_session(session, user_id))
}

/// Ensures the user may invoke code execution for this session (read access, session active).
pub async fn ensure_session_for_execute(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
) -> Result<(), ServiceError> {
  let session = session_repo::find_by_id(pool, session_id)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?
    .ok_or(ServiceError::Repo(RepoError::NotFound))?;

  if !session.is_readable_by(user_id) {
    return Err(ServiceError::SessionPrivate);
  }

  if session.status == SessionStatus::Ended {
    return Err(ServiceError::SessionEnded);
  }

  Ok(())
}

pub async fn update_session_name(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
  req: UpdateSessionNameRequest,
) -> Result<SessionDetailResponse, ServiceError> {
  let name = req.name.trim();
  if name.is_empty() {
    return Err(ServiceError::Validation(
      "name must be between 1 and 255 characters".to_string(),
    ));
  }

  let session = load_session_for_mutation(pool, session_id, user_id).await?;
  let updated = session_repo::update_name(pool, session.id, name)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?;

  Ok(SessionDetailResponse::from_session(updated, user_id))
}

pub async fn update_session_visibility(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
  req: UpdateSessionVisibilityRequest,
) -> Result<SessionDetailResponse, ServiceError> {
  let session = load_session_for_mutation(pool, session_id, user_id).await?;
  let updated = session_repo::update_visibility(pool, session.id, req.visibility)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?;

  Ok(SessionDetailResponse::from_session(updated, user_id))
}

pub async fn end_session(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
) -> Result<SessionDetailResponse, ServiceError> {
  let session = session_repo::find_by_id(pool, session_id)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?
    .ok_or(ServiceError::Repo(RepoError::NotFound))?;

  if !session.is_owned_by(user_id) {
    return Err(ServiceError::Forbidden);
  }

  if session.status == SessionStatus::Ended {
    return Err(ServiceError::SessionAlreadyEnded);
  }

  let updated = session_repo::set_ended(pool, session.id)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?;

  tracing::info!("session ended");
  Ok(SessionDetailResponse::from_session(updated, user_id))
}

async fn load_session_for_mutation(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
) -> Result<Session, ServiceError> {
  let session = session_repo::find_by_id(pool, session_id)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?
    .ok_or(ServiceError::Repo(RepoError::NotFound))?;

  if !session.is_owned_by(user_id) {
    return Err(ServiceError::Forbidden);
  }

  if session.status == SessionStatus::Ended {
    return Err(ServiceError::SessionEnded);
  }

  Ok(session)
}
