use axum::extract::{Path, State};
use serde_json::json;
use uuid::Uuid;

use crate::{
  errors::AppResult,
  extractors::AuthUser,
  response::ApiResponse,
  services::session_service,
  state::AppState,
};

pub async fn execute_code(
  State(state): State<AppState>,
  AuthUser(auth): AuthUser,
  Path(session_id): Path<Uuid>,
) -> AppResult<ApiResponse<serde_json::Value>> {
  session_service::ensure_session_for_execute(&state.db, session_id, auth.id).await?;

  Ok(ApiResponse::ok(json!({
    "stdout": "",
    "stderr": "",
    "status": { "id": null, "description": "stub" }
  })))
}
