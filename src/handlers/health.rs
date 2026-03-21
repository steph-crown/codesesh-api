use serde_json::json;

use crate::{errors::AppResult, response::ApiResponse};

pub async fn health_check() -> AppResult<ApiResponse<serde_json::Value>> {
  Ok(ApiResponse::ok(json!({ "status": "ok" })))
}
