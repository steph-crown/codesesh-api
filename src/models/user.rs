use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
  pub id: Uuid,
  pub display_name: String,
  pub color: String,
  pub created_at: OffsetDateTime,
}
