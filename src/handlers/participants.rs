use axum::extract::{Path, State};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
  dto::ParticipantResponse,
  errors::AppResult,
  response::ApiResponse,
  state::AppState,
};

pub async fn list_participants(
  State(_state): State<AppState>,
  Path(_session_id): Path<Uuid>,
) -> AppResult<ApiResponse<Vec<ParticipantResponse>>> {
  Ok(ApiResponse::ok(vec![]))
}

pub async fn create_participant(
  State(_state): State<AppState>,
  Path(_session_id): Path<Uuid>,
) -> AppResult<ApiResponse<ParticipantResponse>> {
  Ok(ApiResponse::created(ParticipantResponse {
    user_id: Uuid::nil(),
    display_name: String::new(),
    joined_at: OffsetDateTime::UNIX_EPOCH,
    is_active: true,
  }))
}
