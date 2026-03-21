use axum::extract::{Path, State};
use serde_json::json;

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
  Path(short_id): Path<String>,
) -> AppResult<ApiResponse<serde_json::Value>> {
  let session_id = session_service::resolve_session_id(&state.db, &short_id).await?;
  session_service::ensure_session_for_execute(&state.db, session_id, auth.id).await?;

  Ok(ApiResponse::ok(json!({
    "stdout": "",
    "stderr": "",
    "status": { "id": null, "description": "stub" }
  })))
}
