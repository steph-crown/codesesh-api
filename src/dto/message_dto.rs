// src/dto/message_dto.rs
use crate::models::ChatMessage;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

// ─── Request ──────────────────────────────────────────────────────────────────

// Note: chat messages are sent via WebSocket, not REST
// This DTO is only used for the one-time history fetch on join
// No create request DTO needed here — creation happens in the WS handler

// ─── Response ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ChatMessageResponse {
  pub id: Uuid,
  pub session_id: Uuid,
  pub user_id: Uuid,
  pub display_name: String, // joined from users table — not on ChatMessage model
  pub content: String,
  pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct MessageHistoryResponse {
  pub messages: Vec<ChatMessageResponse>,
  pub has_more: bool, // whether there are older messages before this batch
}
