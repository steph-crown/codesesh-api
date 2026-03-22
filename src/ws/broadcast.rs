//! Only module that writes to WebSocket sinks (besides targeted sends in `ws_service`).

use axum::extract::ws::Message;
use futures_util::SinkExt;
use uuid::Uuid;

use crate::state::ActiveSession;
use crate::ws::messages::ServerMessage;

fn serialize(msg: &ServerMessage) -> Option<String> {
  match serde_json::to_string(msg) {
    Ok(s) => Some(s),
    Err(e) => {
      tracing::warn!(error = %e, "serialize ws ServerMessage failed");
      None
    }
  }
}

/// Send to all participants except one (e.g. text_change, cursor_move).
pub async fn broadcast_except(
  session: &mut ActiveSession,
  exclude_user_id: Uuid,
  msg: &ServerMessage,
) {
  let Some(json) = serialize(msg) else {
    return;
  };
  let text = json;
  for p in session.participants.iter_mut() {
    if p.user_id == exclude_user_id {
      continue;
    }
    if let Err(e) = p.sender.send(Message::text(text.clone())).await {
      tracing::warn!(
        error = %e,
        user_id = %p.user_id,
        "failed to send ws message to participant"
      );
    }
  }
}

/// Send to all participants including sender (chat, language, joins, leaves, session_ended).
pub async fn broadcast_all(session: &mut ActiveSession, msg: &ServerMessage) {
  let Some(json) = serialize(msg) else {
    return;
  };
  let text = json;
  for p in session.participants.iter_mut() {
    if let Err(e) = p.sender.send(Message::text(text.clone())).await {
      tracing::warn!(
        error = %e,
        user_id = %p.user_id,
        "failed to send ws message to participant"
      );
    }
  }
}

/// Send to exactly one participant (full_sync, errors).
pub async fn send_to(session: &mut ActiveSession, user_id: Uuid, msg: &ServerMessage) {
  let Some(json) = serialize(msg) else {
    return;
  };
  let text = json;
  for p in session.participants.iter_mut() {
    if p.user_id == user_id {
      if let Err(e) = p.sender.send(Message::text(text)).await {
        tracing::warn!(
          error = %e,
          user_id = %p.user_id,
          "failed to send ws message to participant"
        );
      }
      break;
    }
  }
}
