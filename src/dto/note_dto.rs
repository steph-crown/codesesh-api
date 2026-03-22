use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct UpsertNoteRequest {
  #[validate(length(max = 100_000))]
  pub content: String,
}

#[derive(Debug, Serialize)]
pub struct NoteResponse {
  pub id: Option<Uuid>,
  pub session_id: Uuid,
  pub user_id: Uuid,
  pub content: String,
  pub updated_at: Option<OffsetDateTime>,
}

impl NoteResponse {
  pub fn empty(session_id: Uuid, user_id: Uuid) -> Self {
    Self {
      id: None,
      session_id,
      user_id,
      content: String::new(),
      updated_at: None,
    }
  }

  pub fn from_note(note: crate::models::SessionNote) -> Self {
    Self {
      id: Some(note.id),
      session_id: note.session_id,
      user_id: note.user_id,
      content: note.content,
      updated_at: Some(note.updated_at),
    }
  }
}
