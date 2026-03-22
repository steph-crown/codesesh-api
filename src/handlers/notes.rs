use axum::extract::{Path, State};
use validator::Validate;

use crate::{
  dto::{NoteResponse, UpsertNoteRequest},
  errors::{AppError, AppResult},
  extractors::{AppJson, AuthUser},
  response::ApiResponse,
  services::{note_service, session_service},
  state::AppState,
};

pub async fn get_note(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  Path(short_id): Path<String>,
) -> AppResult<ApiResponse<NoteResponse>> {
  let session_id = session_service::resolve_session_id(&state.db, &short_id).await?;
  let res = note_service::get_note(&state.db, session_id, auth.id).await?;
  Ok(ApiResponse::ok(res))
}

pub async fn upsert_note(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  Path(short_id): Path<String>,
  AppJson(payload): AppJson<UpsertNoteRequest>,
) -> AppResult<ApiResponse<NoteResponse>> {
  payload
    .validate()
    .map_err(|e| AppError::Validation(e.to_string()))?;

  let session_id = session_service::resolve_session_id(&state.db, &short_id).await?;
  let res =
    note_service::upsert_note(&state.db, session_id, auth.id, payload.content).await?;
  Ok(ApiResponse::ok(res))
}
