use axum::extract::State;
use validator::Validate;

use crate::{
  dto::{CreateUserRequest, UserResponse},
  errors::{AppError, AppResult},
  extractors::{AppJson, AuthUser},
  response::ApiResponse,
  services::user_service,
  state::AppState,
};

pub async fn create_user(
  State(state): State<AppState>,
  AppJson(payload): AppJson<CreateUserRequest>,
) -> AppResult<ApiResponse<UserResponse>> {
  payload
    .validate()
    .map_err(|e| AppError::Validation(e.to_string()))?;

  let user = user_service::create_user(&state.db, payload).await?;

  Ok(ApiResponse::created(UserResponse::from(user)))
}

pub async fn get_current_user(AuthUser(user): AuthUser) -> AppResult<ApiResponse<UserResponse>> {
  Ok(ApiResponse::ok(UserResponse::from(user)))
}
