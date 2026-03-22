use sqlx::PgPool;
use uuid::Uuid;

use crate::{
  dto::NoteResponse,
  errors::{RepoError, ServiceError},
  repositories::{note_repo, session_repo},
};

pub async fn get_note(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
) -> Result<NoteResponse, ServiceError> {
  let session = session_repo::find_by_id(pool, session_id)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?
    .ok_or(ServiceError::Repo(RepoError::NotFound))?;

  if !session.is_readable_by(user_id) {
    return Err(ServiceError::SessionPrivate);
  }

  let row = note_repo::find_by_session_and_user(pool, session_id, user_id)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?;

  Ok(match row {
    Some(note) => NoteResponse::from_note(note),
    None => NoteResponse::empty(session_id, user_id),
  })
}

pub async fn upsert_note(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
  content: String,
) -> Result<NoteResponse, ServiceError> {
  let session = session_repo::find_by_id(pool, session_id)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?
    .ok_or(ServiceError::Repo(RepoError::NotFound))?;

  if !session.is_readable_by(user_id) {
    return Err(ServiceError::SessionPrivate);
  }

  let note = note_repo::upsert(pool, session_id, user_id, &content)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?;

  Ok(NoteResponse::from_note(note))
}
