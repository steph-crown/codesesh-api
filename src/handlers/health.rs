use axum::Json;
use serde_json::json;

use crate::errors::AppResult;

pub async fn health_check() -> AppResult<Json<serde_json::Value>> {
  Ok(Json(json!({ "status": "ok" })))
}
