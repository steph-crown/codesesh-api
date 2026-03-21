use axum::{Json, extract::State};
use serde_json::json;
use uuid::Uuid;

use crate::{
  dto::{CreateUserRequest, CreateUserResponse},
  errors::AppResult,
  state::AppState,
};

pub async fn create_user(
  State(state): State<AppState>,
  Json(user): Json<CreateUserRequest>,
) -> AppResult<Json<CreateUserResponse>> {
  Ok(Json(CreateUserResponse {
    id: Uuid::new_v4(),
    display_name: "Meee".to_string(),
  }))
}
