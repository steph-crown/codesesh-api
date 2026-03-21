use axum::extract::{Path, Query, State};
use uuid::Uuid;

use crate::{
  dto::{GetMessagesQuery, MessageHistoryResponse},
  errors::AppResult,
  state::AppState,
};
use axum::Json;

pub async fn list_messages(
  State(_state): State<AppState>,
  Path(_session_id): Path<Uuid>,
  Query(_query): Query<GetMessagesQuery>,
) -> AppResult<Json<MessageHistoryResponse>> {
  Ok(Json(MessageHistoryResponse {
    messages: vec![],
    has_more: false,
  }))
}
