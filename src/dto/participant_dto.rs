use crate::models::SessionParticipant;
use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

// ─── Response DTOs ────────────────────────────────────────────────────────────

/// Returned when listing participants in a session
#[derive(Debug, Serialize)]
pub struct ParticipantResponse {
  pub user_id: Uuid,
  pub display_name: String,
  pub joined_at: OffsetDateTime,
  pub is_active: bool,
}

// ─── Constructors ─────────────────────────────────────────────────────────────

impl ParticipantResponse {
  pub fn from_participant(participant: SessionParticipant, display_name: String) -> Self {
    Self {
      user_id: participant.user_id,
      display_name,
      joined_at: participant.joined_at,
      is_active: participant.is_active(),
    }
  }
}
