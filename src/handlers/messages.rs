use axum::extract::{Path, Query, State};
use uuid::Uuid;
use validator::Validate;

use crate::{
  dto::{GetMessagesQuery, MessageHistoryResponse},
  errors::{AppError, AppResult},
  extractors::AuthUser,
  response::ApiResponse,
  services::message_service,
  state::AppState,
};

pub async fn list_messages(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  Path(session_id): Path<Uuid>,
  Query(query): Query<GetMessagesQuery>,
) -> AppResult<ApiResponse<MessageHistoryResponse>> {
  query
    .validate()
    .map_err(|e| AppError::Validation(e.to_string()))?;

  let res = message_service::list_messages(&state.db, session_id, auth.id, query).await?;
  Ok(ApiResponse::ok(res))
}
