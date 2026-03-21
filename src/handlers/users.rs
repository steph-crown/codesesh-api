use axum::{Json, extract::State};
use uuid::Uuid;

use crate::{
  dto::{CreateUserRequest, CreateUserResponse},
  errors::AppResult,
  state::AppState,
};

pub async fn create_user(
  State(_state): State<AppState>,
  Json(payload): Json<CreateUserRequest>,
) -> AppResult<Json<CreateUserResponse>> {
  Ok(Json(CreateUserResponse {
    id: Uuid::nil(),
    display_name: payload.display_name,
  }))
}
