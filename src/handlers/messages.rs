use axum::extract::{Path, Query, State};
use validator::Validate;

use crate::{
  dto::{GetMessagesQuery, MessageHistoryResponse},
  errors::{AppError, AppResult},
  extractors::AuthUser,
  response::ApiResponse,
  services::{message_service, session_service},
  state::AppState,
};

pub async fn list_messages(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  Path(short_id): Path<String>,
  Query(query): Query<GetMessagesQuery>,
) -> AppResult<ApiResponse<MessageHistoryResponse>> {
  query
    .validate()
    .map_err(|e| AppError::Validation(e.to_string()))?;

  let session_id = session_service::resolve_session_id(&state.db, &short_id).await?;
  let res = message_service::list_messages(&state.db, session_id, auth.id, query).await?;
  Ok(ApiResponse::ok(res))
}
