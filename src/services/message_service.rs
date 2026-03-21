use sqlx::PgPool;
use uuid::Uuid;

use crate::{
  dto::{ChatMessageResponse, GetMessagesQuery, MessageHistoryResponse},
  errors::{RepoError, ServiceError},
  repositories::{message_repo, session_repo},
};

pub async fn list_messages(
  pool: &PgPool,
  session_id: Uuid,
  user_id: Uuid,
  query: GetMessagesQuery,
) -> Result<MessageHistoryResponse, ServiceError> {
  let session = session_repo::find_by_id(pool, session_id)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?
    .ok_or(ServiceError::Repo(RepoError::NotFound))?;

  if !session.is_readable_by(user_id) {
    return Err(ServiceError::SessionPrivate);
  }

  let limit = query.limit.unwrap_or(50).clamp(1, 100);
  let (rows, has_more) = message_repo::list_history(pool, session_id, limit, query.before)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?;

  let messages = rows
    .into_iter()
    .map(|r| ChatMessageResponse {
      id: r.id,
      session_id: r.session_id,
      user_id: r.user_id,
      display_name: r.display_name,
      content: r.content,
      created_at: r.created_at,
    })
    .collect();

  Ok(MessageHistoryResponse { messages, has_more })
}
