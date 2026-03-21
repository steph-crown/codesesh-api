use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "session_visibility", rename_all = "snake_case")]
pub enum SessionVisibility {
  Private,
  ViewOnly,
  Edit,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "session_status", rename_all = "snake_case")]
pub enum SessionStatus {
  Active,
  Ended,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
  pub id: Uuid,
  pub short_id: String,
  pub host_id: Uuid,
  pub name: String,
  pub language: String,
  pub visibility: SessionVisibility,
  pub status: SessionStatus,
  pub content: String,
  pub last_activity_at: OffsetDateTime,
  pub event_count: i32, // will be used post mvp for session events.
  pub created_at: OffsetDateTime,
  pub updated_at: OffsetDateTime,
}

impl Session {
  pub fn is_ended(&self) -> bool {
    self.status == SessionStatus::Ended
  }

  pub fn is_owned_by(&self, user_id: Uuid) -> bool {
    self.host_id == user_id
  }

  pub fn is_editable_by(&self, user_id: Uuid) -> bool {
    match self.visibility {
      SessionVisibility::Edit => true,
      SessionVisibility::ViewOnly => self.host_id == user_id,
      SessionVisibility::Private => self.host_id == user_id,
    }
  }

  pub fn is_readable_by(&self, user_id: Uuid) -> bool {
    match self.visibility {
      SessionVisibility::Private => self.host_id == user_id,
      SessionVisibility::ViewOnly => true,
      SessionVisibility::Edit => true,
    }
  }
}
