use axum::extract::State;
use uuid::Uuid;

use crate::{
  dto::{CreateUserRequest, CreateUserResponse},
  errors::AppResult,
  extractors::AppJson,
  response::ApiResponse,
  state::AppState,
};

pub async fn create_user(
  State(_state): State<AppState>,
  AppJson(payload): AppJson<CreateUserRequest>,
) -> AppResult<ApiResponse<CreateUserResponse>> {
  Ok(ApiResponse::created(CreateUserResponse {
    id: Uuid::nil(),
    display_name: payload.display_name,
  }))
}
