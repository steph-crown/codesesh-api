use axum::extract::{Path, Query, State};
use uuid::Uuid;

use crate::{
  dto::{GetMessagesQuery, MessageHistoryResponse},
  errors::AppResult,
  response::ApiResponse,
  state::AppState,
};

pub async fn list_messages(
  State(_state): State<AppState>,
  Path(_session_id): Path<Uuid>,
  Query(_query): Query<GetMessagesQuery>,
) -> AppResult<ApiResponse<MessageHistoryResponse>> {
  Ok(ApiResponse::ok(MessageHistoryResponse {
    messages: vec![],
    has_more: false,
  }))
}
