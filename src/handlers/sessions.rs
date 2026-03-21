use axum::{Json, extract::State};

use crate::{
  dto::{CreateSessionRequest, SessionDetailResponse},
  errors::AppResult,
  state::AppState,
};

pub async fn create_session(
  State(state): State<AppState>,
  Json(session): Json<CreateSessionRequest>,
) -> AppResult<Json<SessionDetailResponse>> {
  Ok(SessionDetailResponse {})
}
