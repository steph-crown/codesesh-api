use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SessionParticipant {
  pub id: Uuid,
  pub session_id: Uuid,
  pub user_id: Uuid,
  pub joined_at: OffsetDateTime,
  pub left_at: Option<OffsetDateTime>,
}

impl SessionParticipant {
  pub fn is_active(&self) -> bool {
    self.left_at.is_none()
  }
}
