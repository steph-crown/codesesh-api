use axum::{Json, extract::State};
use uuid::Uuid;

use crate::{
  dto::{CreateUserRequest, CreateUserResponse},
  errors::AppResult,
  response::ApiResponse,
  state::AppState,
};

pub async fn create_user(
  State(_state): State<AppState>,
  Json(payload): Json<CreateUserRequest>,
) -> AppResult<ApiResponse<CreateUserResponse>> {
  Ok(ApiResponse::created(CreateUserResponse {
    id: Uuid::nil(),
    display_name: payload.display_name,
  }))
}
