use crate::models::User;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
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

  #[validate(length(
    min = 1,
    max = 64,
    message = "color must be between 1 and 64 characters"
  ))]
  pub color: String,
}

// ─── Response ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct UserResponse {
  pub id: Uuid,
  pub display_name: String,
  pub color: String,
  pub created_at: OffsetDateTime,
}

impl From<User> for UserResponse {
  fn from(user: User) -> Self {
    Self {
      id: user.id,
      display_name: user.display_name,
      color: user.color,
      created_at: user.created_at,
    }
  }
}
