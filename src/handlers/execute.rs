use axum::{Json, extract::Path};
use serde_json::json;
use uuid::Uuid;

use crate::{errors::AppResult, state::AppState};
use axum::extract::State;

pub async fn execute_code(
  State(_state): State<AppState>,
  Path(_session_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
  Ok(Json(json!({
    "stdout": "",
    "stderr": "",
    "status": { "id": null, "description": "stub" }
  })))
}
