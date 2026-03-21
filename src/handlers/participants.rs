use axum::{
  Json,
  extract::{Path, State},
};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{dto::ParticipantResponse, errors::AppResult, state::AppState};

pub async fn list_participants(
  State(_state): State<AppState>,
  Path(_session_id): Path<Uuid>,
) -> AppResult<Json<Vec<ParticipantResponse>>> {
  Ok(Json(vec![]))
}

pub async fn create_participant(
  State(_state): State<AppState>,
  Path(_session_id): Path<Uuid>,
) -> AppResult<Json<ParticipantResponse>> {
  Ok(Json(ParticipantResponse {
    user_id: Uuid::nil(),
    display_name: String::new(),
    joined_at: OffsetDateTime::UNIX_EPOCH,
    is_active: true,
  }))
}
