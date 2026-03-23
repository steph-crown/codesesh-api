use axum::extract::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  errors::AppResult,
  extractors::AppJson,
  models::SessionLanguage,
  repositories::run_repo,
  response::ApiResponse,
  state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
  pub session_id: Uuid,
  pub user_id: Uuid,
  pub language: SessionLanguage,
}

#[derive(Debug, Serialize)]
pub struct RunCreatedResponse {
  pub id: Uuid,
}

pub async fn create_run(
  State(state): State<AppState>,
  AppJson(payload): AppJson<CreateRunRequest>,
) -> AppResult<ApiResponse<RunCreatedResponse>> {
  let id =
    run_repo::insert(&state.db, payload.session_id, payload.user_id, payload.language).await?;

  Ok(ApiResponse::created(RunCreatedResponse { id }))
}
