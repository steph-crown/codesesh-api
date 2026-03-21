use axum::extract::{Path, State};
use uuid::Uuid;

use crate::{
  dto::ParticipantResponse,
  errors::AppResult,
  extractors::AuthUser,
  response::ApiResponse,
  services::participant_service,
  state::AppState,
};

pub async fn list_participants(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  Path(session_id): Path<Uuid>,
) -> AppResult<ApiResponse<Vec<ParticipantResponse>>> {
  let res = participant_service::list_participants(&state.db, session_id, auth.id).await?;
  Ok(ApiResponse::ok(res))
}

pub async fn create_participant(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  Path(session_id): Path<Uuid>,
) -> AppResult<ApiResponse<ParticipantResponse>> {
  let res = participant_service::join_session(&state.db, session_id, auth.id).await?;
  Ok(ApiResponse::created(res))
}
