use axum::extract::{Path, State};

use crate::{
  dto::{ParticipantResponse, SessionParticipationResponse},
  errors::AppResult,
  extractors::AuthUser,
  response::ApiResponse,
  services::{participant_service, session_service},
  state::AppState,
};

pub async fn list_participants(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  Path(short_id): Path<String>,
) -> AppResult<ApiResponse<Vec<ParticipantResponse>>> {
  let session_id = session_service::resolve_session_id(&state.db, &short_id).await?;
  let res = participant_service::list_participants(&state.db, session_id, auth.id).await?;
  Ok(ApiResponse::ok(res))
}

pub async fn create_participant(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  Path(short_id): Path<String>,
) -> AppResult<ApiResponse<ParticipantResponse>> {
  let session_id = session_service::resolve_session_id(&state.db, &short_id).await?;
  let res = participant_service::join_session(&state.db, session_id, auth.id).await?;
  Ok(ApiResponse::created(res))
}

pub async fn get_participation(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  Path(short_id): Path<String>,
) -> AppResult<ApiResponse<SessionParticipationResponse>> {
  let session_id = session_service::resolve_session_id(&state.db, &short_id).await?;
  let res =
    participant_service::participation_status(&state.db, session_id, auth.id).await?;
  Ok(ApiResponse::ok(res))
}
