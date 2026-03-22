use axum::extract::{Path, Query, State};
use validator::Validate;

use crate::{
  dto::{
    CreateSessionRequest, GetSessionsQuery, PaginatedResponse, SessionDetailResponse,
    SessionSummaryResponse, UpdateSessionNameRequest, UpdateSessionVisibilityRequest,
  },
  errors::{AppError, AppResult},
  extractors::{AppJson, AuthUser},
  response::ApiResponse,
  services::{session_service, ws_service},
  state::AppState,
};
use crate::ws::messages::SessionEndReason;

pub async fn create_session(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  AppJson(payload): AppJson<CreateSessionRequest>,
) -> AppResult<ApiResponse<SessionDetailResponse>> {
  payload
    .validate()
    .map_err(|e| AppError::Validation(e.to_string()))?;

  let session = session_service::create_session(&state.db, auth.id, payload).await?;

  Ok(ApiResponse::created(SessionDetailResponse::from_session(
    session,
    auth.id,
  )))
}

pub async fn list_sessions(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  Query(query): Query<GetSessionsQuery>,
) -> AppResult<ApiResponse<PaginatedResponse<SessionSummaryResponse>>> {
  let res = session_service::list_sessions(&state.db, auth.id, query).await?;
  Ok(ApiResponse::ok(res))
}

pub async fn get_session(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  Path(short_id): Path<String>,
) -> AppResult<ApiResponse<SessionDetailResponse>> {
  let session_id = session_service::resolve_session_id(&state.db, &short_id).await?;
  let res = session_service::get_session(&state.db, session_id, auth.id).await?;
  Ok(ApiResponse::ok(res))
}

pub async fn update_session_name(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  Path(short_id): Path<String>,
  AppJson(payload): AppJson<UpdateSessionNameRequest>,
) -> AppResult<ApiResponse<SessionDetailResponse>> {
  payload
    .validate()
    .map_err(|e| AppError::Validation(e.to_string()))?;

  let session_id = session_service::resolve_session_id(&state.db, &short_id).await?;
  let res =
    session_service::update_session_name(&state.db, session_id, auth.id, payload).await?;

  Ok(ApiResponse::ok(res))
}

pub async fn update_session_visibility(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  Path(short_id): Path<String>,
  AppJson(payload): AppJson<UpdateSessionVisibilityRequest>,
) -> AppResult<ApiResponse<SessionDetailResponse>> {
  let session_id = session_service::resolve_session_id(&state.db, &short_id).await?;
  let res =
    session_service::update_session_visibility(&state.db, session_id, auth.id, payload).await?;

  ws_service::sync_active_session_visibility(&state, res.id, res.visibility.clone());

  Ok(ApiResponse::ok(res))
}

pub async fn end_session(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  Path(short_id): Path<String>,
) -> AppResult<ApiResponse<SessionDetailResponse>> {
  let session_id = session_service::resolve_session_id(&state.db, &short_id).await?;
  let res = session_service::end_session(&state.db, session_id, auth.id).await?;
  ws_service::broadcast_session_ended(&state, session_id, SessionEndReason::HostEnded).await;
  Ok(ApiResponse::ok(res))
}
