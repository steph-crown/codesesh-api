use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

/// Buffered text-change events before DB flush (content + event_count update).
pub struct BufferedEvent {
  pub payload: Value,
  pub actor_user_id: Uuid,
  pub offset_ms: i64,
  pub created_at: OffsetDateTime,
}

pub struct EventBuffer {
  pub session_id: Uuid,
  events: Vec<BufferedEvent>,
}

impl EventBuffer {
  pub fn new(session_id: Uuid) -> Self {
    Self {
      session_id,
      events: Vec::new(),
    }
  }

  pub fn push(&mut self, event: BufferedEvent) {
    self.events.push(event);
  }

  pub fn drain(&mut self) -> Vec<BufferedEvent> {
    std::mem::take(&mut self.events)
  }

  pub fn is_empty(&self) -> bool {
    self.events.is_empty()
  }

  pub fn len(&self) -> usize {
    self.events.len()
  }
}
