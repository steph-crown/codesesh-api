use crate::models::User;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ─── Request ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
  #[validate(length(
    min = 1,
    max = 100,
    message = "display name must be between 1 and 100 characters"
  ))]
  pub display_name: String,
}

// ─── Response ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
  pub id: Uuid,
  pub display_name: String,
}

impl From<User> for CreateUserResponse {
  fn from(user: User) -> Self {
    Self {
      id: user.id,
      display_name: user.display_name,
    }
  }
}
