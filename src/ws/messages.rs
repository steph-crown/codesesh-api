//! WebSocket wire types (JSON). Kept in `ws/`; business logic lives in `services/ws_service`.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::SessionLanguage;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
  TextChange(TextChangeDelta),
  CursorMove(CursorPosition),
  ChatMessage(ChatContent),
  LanguageChange(LanguagePayload),
  Leave,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
  FullSync(FullSyncPayload),
  TextChange(TextChangePayload),
  CursorMove(CursorPayload),
  ChatMessage(ChatMessagePayload),
  LanguageChange(LanguageChangePayload),
  ParticipantJoin(ParticipantPayload),
  ParticipantLeave(ParticipantLeavePayload),
  SessionEnded(SessionEndedPayload),
  Error(WsErrorPayload),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TextChangeDelta {
  pub range: EditorRange,
  pub text: String,
  pub version: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EditorRange {
  pub start_line: u32,
  pub start_column: u32,
  pub end_line: u32,
  pub end_column: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CursorPosition {
  pub line: u32,
  pub column: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChatContent {
  pub content: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LanguagePayload {
  pub language: SessionLanguage,
}

#[derive(Debug, Serialize, Clone)]
pub struct FullSyncPayload {
  pub content: String,
  pub version: u64,
  pub language: SessionLanguage,
  pub participants: Vec<ParticipantInfo>,
  pub messages: Vec<ChatMessagePayload>,
  pub is_owner: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct TextChangePayload {
  pub delta: TextChangeDelta,
  pub version: u64,
  pub user_id: Uuid,
  pub display_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct CursorPayload {
  pub line: u32,
  pub column: u32,
  pub user_id: Uuid,
  pub display_name: String,
  pub color: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ChatMessagePayload {
  pub id: Uuid,
  pub content: String,
  pub user_id: Uuid,
  pub display_name: String,
  pub color: String,
  pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Clone)]
pub struct LanguageChangePayload {
  pub language: SessionLanguage,
  pub user_id: Uuid,
  pub display_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ParticipantInfo {
  pub user_id: Uuid,
  pub display_name: String,
  pub color: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ParticipantPayload {
  pub user_id: Uuid,
  pub display_name: String,
  pub color: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ParticipantLeavePayload {
  pub user_id: Uuid,
  pub display_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SessionEndedPayload {
  pub reason: SessionEndReason,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionEndReason {
  HostEnded,
  IdleTimeout,
  EventCapReached,
}

#[derive(Debug, Serialize, Clone)]
pub struct WsErrorPayload {
  pub code: String,
  pub message: String,
}
