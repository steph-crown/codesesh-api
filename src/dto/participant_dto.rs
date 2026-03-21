use crate::models::SessionParticipant;
use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

// ─── Response DTOs ────────────────────────────────────────────────────────────

/// `GET /sessions/:short_id/participation` — whether `X-User-Id` counts as a participant
/// (host always does; others need an active `session_participants` row).
#[derive(Debug, Serialize)]
pub struct SessionParticipationResponse {
  pub is_participant: bool,
}

/// Returned when listing participants in a session
#[derive(Debug, Serialize)]
pub struct ParticipantResponse {
  pub user_id: Uuid,
  pub display_name: String,
  pub color: String,
  pub joined_at: OffsetDateTime,
  pub is_active: bool,
}

// ─── Constructors ─────────────────────────────────────────────────────────────

impl ParticipantResponse {
  pub fn from_participant(
    participant: SessionParticipant,
    display_name: String,
    color: String,
  ) -> Self {
    Self {
      user_id: participant.user_id,
      display_name,
      color,
      joined_at: participant.joined_at,
      is_active: participant.is_active(),
    }
  }
}
