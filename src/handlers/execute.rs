use axum::extract::{Path, State};
use serde_json::json;
use uuid::Uuid;

use crate::{errors::AppResult, response::ApiResponse, state::AppState};

pub async fn execute_code(
  State(_state): State<AppState>,
  Path(_session_id): Path<Uuid>,
) -> AppResult<ApiResponse<serde_json::Value>> {
  Ok(ApiResponse::ok(json!({
    "stdout": "",
    "stderr": "",
    "status": { "id": null, "description": "stub" }
  })))
}
