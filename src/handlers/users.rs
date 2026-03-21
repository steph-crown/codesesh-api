use axum::extract::State;
use validator::Validate;

use crate::{
  dto::{CreateUserRequest, CreateUserResponse},
  errors::{AppError, AppResult},
  extractors::AppJson,
  response::ApiResponse,
  services::user_service,
  state::AppState,
};

pub async fn create_user(
  State(state): State<AppState>,
  AppJson(payload): AppJson<CreateUserRequest>,
) -> AppResult<ApiResponse<CreateUserResponse>> {
  payload
    .validate()
    .map_err(|e| AppError::Validation(e.to_string()))?;

  let user = user_service::create_user(&state.db, payload).await?;

  Ok(ApiResponse::created(CreateUserResponse::from(user)))
}
