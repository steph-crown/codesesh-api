use sqlx::PgPool;
use uuid::Uuid;

use crate::{
  dto::ParticipantResponse,
  errors::{RepoError, ServiceError},
  models::SessionStatus,
  repositories::{participant_repo, session_repo, user_repo},
};

pub async fn list_participants(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
) -> Result<Vec<ParticipantResponse>, ServiceError> {
  let session = session_repo::find_by_id(pool, session_id)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?
    .ok_or(ServiceError::Repo(RepoError::NotFound))?;

  if !session.is_readable_by(user_id) {
    return Err(ServiceError::SessionPrivate);
  }

  let rows = participant_repo::list_with_display_names(pool, session_id)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?;

  Ok(rows
    .into_iter()
    .map(|(p, name, color)| ParticipantResponse::from_participant(p, name, color))
    .collect())
}

pub async fn join_session(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
) -> Result<ParticipantResponse, ServiceError> {
  let session = session_repo::find_by_id(pool, session_id)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?
    .ok_or(ServiceError::Repo(RepoError::NotFound))?;

  if session.status == SessionStatus::Ended {
    return Err(ServiceError::SessionEnded);
  }

  if !session.is_readable_by(user_id) {
    return Err(ServiceError::SessionPrivate);
  }

  let participant = participant_repo::upsert_active(pool, session_id, user_id)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?;

  let user = user_repo::find_by_id(pool, user_id)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?
    .ok_or(ServiceError::Repo(RepoError::NotFound))?;

  tracing::info!("session join recorded");
  Ok(ParticipantResponse::from_participant(
    participant,
    user.display_name,
    user.color,
  ))
}
