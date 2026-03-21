use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
  pub id: Uuid,
  pub display_name: String,
  pub created_at: OffsetDateTime,
}
