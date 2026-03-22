//! WebSocket session lifecycle and message handling.

use std::time::{Duration, Instant};

use axum::extract::ws::{Message, WebSocket};
use futures_util::StreamExt;
use serde_json::json;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::{Session, SessionStatus, User};
use crate::repositories::{message_repo, participant_repo, session_repo, user_repo};
use crate::state::{ActiveParticipant, ActiveSession, AppState};
use crate::ws::broadcast;
use crate::ws::event_buffer::BufferedEvent;
use crate::ws::messages::{
  ChatContent, ChatMessagePayload, ClientMessage, CursorPayload, CursorPosition, FullSyncPayload,
  LanguageChangePayload, LanguagePayload, ParticipantInfo, ParticipantLeavePayload,
  ParticipantPayload, ServerMessage, SessionEndReason, SessionEndedPayload, TextChangeDelta,
  TextChangePayload, WsErrorPayload,
};
use crate::ws::text_edit::apply_text_delta;

const MAX_CHAT_CHARS: usize = 2000;
const EVENT_CAP: i32 = 100_000;
const IDLE_TIMEOUT: Duration = Duration::from_secs(30 * 60);

fn new_active_session(s: &Session) -> ActiveSession {
  ActiveSession {
    session_id: s.id,
    host_id: s.host_id,
    session_created_at: s.created_at,
    content: s.content.clone(),
    version: 0,
    language: s.language.clone(),
    visibility: s.visibility.clone(),
    participants: Vec::new(),
    last_event_at: Instant::now(),
    published_event_count: s.event_count,
    event_buffer: std::sync::Mutex::new(crate::ws::event_buffer::EventBuffer::new(s.id)),
  }
}

fn ws_err(code: &str, message: &str) -> ServerMessage {
  ServerMessage::Error(WsErrorPayload {
    code: code.to_string(),
    message: message.to_string(),
  })
}

fn chat_row_to_payload(m: &message_repo::MessageWithAuthor) -> ChatMessagePayload {
  ChatMessagePayload {
    id: m.id,
    content: m.content.clone(),
    user_id: m.user_id,
    display_name: m.display_name.clone(),
    color: m.color.clone(),
    created_at: m.created_at,
  }
}

/// Public entry: run after HTTP upgrade and validations.
pub async fn handle_connection(
  socket: WebSocket,
  session_row: Session,
  user: User,
  state: AppState,
  is_owner: bool,
) {
  let (sender, mut receiver) = socket.split();
  let session_id = session_row.id;

  let first_in_session = {
    let mut entry = state
      .sessions
      .entry(session_id)
      .or_insert_with(|| new_active_session(&session_row));
    let first = entry.participants.is_empty();
    entry.participants.push(ActiveParticipant {
      user_id: user.id,
      display_name: user.display_name.clone(),
      color: user.color.clone(),
      sender,
    });
    first
  };

  if first_in_session {
    spawn_flush_loop(state.clone(), session_id);
  }

  tracing::info!(
    session_id = %session_id,
    user_id = %user.id,
    "websocket participant connected"
  );

  if let Err(e) = send_full_sync(&state, session_id, &user, is_owner).await {
    tracing::error!(error = %e, "full_sync failed");
  }

  {
    let join = ServerMessage::ParticipantJoin(ParticipantPayload {
      user_id: user.id,
      display_name: user.display_name.clone(),
      color: user.color.clone(),
    });
    if let Some(mut ent) = state.sessions.get_mut(&session_id) {
      broadcast::broadcast_except(&mut ent, user.id, &join).await;
    }
  }

  loop {
    let next = receiver.next().await;
    match next {
      Some(Ok(Message::Text(text))) => {
        match serde_json::from_str::<ClientMessage>(&text) {
          Ok(msg) => {
            let done = handle_client_message(&state, session_id, &user, msg).await;
            if done {
              break;
            }
          }
          Err(_) => {
            if let Some(mut ent) = state.sessions.get_mut(&session_id) {
              let err = ws_err("INVALID_MESSAGE", "Invalid message format");
              broadcast::send_to(&mut ent, user.id, &err).await;
            }
          }
        }
      }
      Some(Ok(Message::Close(_))) | None => {
        handle_disconnect(&state, session_id, user.id, &user.display_name).await;
        break;
      }
      Some(Err(e)) => {
        tracing::warn!(
          error = %e,
          session_id = %session_id,
          user_id = %user.id,
          "websocket receive error"
        );
        handle_disconnect(&state, session_id, user.id, &user.display_name).await;
        break;
      }
      _ => {}
    }
  }
}

/// Broadcast session end and drop in-memory session (REST idle/cap/host end).
pub async fn broadcast_session_ended(state: &AppState, session_id: Uuid, reason: SessionEndReason) {
  if let Some(ent) = state.sessions.get(&session_id) {
    let drained: Vec<BufferedEvent> = ent.event_buffer.lock().unwrap().drain();
    if !drained.is_empty() {
      let n = drained.len() as i32;
      let content = ent.content.clone();
      if let Err(e) =
        session_repo::apply_content_and_increment_events(&state.db, session_id, &content, n).await
      {
        tracing::warn!(error = %e, "final flush before session end failed");
      } else if let Some(mut e) = state.sessions.get_mut(&session_id) {
        e.published_event_count += n;
      }
    }
  }
  let msg = ServerMessage::SessionEnded(SessionEndedPayload { reason });
  if let Some(mut ent) = state.sessions.get_mut(&session_id) {
    broadcast::broadcast_all(&mut ent, &msg).await;
  }
  state.sessions.remove(&session_id);
  tracing::info!(session_id = %session_id, ?reason, "session ended broadcast");
}

async fn send_full_sync(
  state: &AppState,
  session_id: Uuid,
  user: &User,
  is_owner: bool,
) -> Result<(), sqlx::Error> {
  let (history, _) = message_repo::list_history(&state.db, session_id, 50, None).await?;
  let messages: Vec<ChatMessagePayload> = history.iter().map(chat_row_to_payload).collect();

  let participants_rows = participant_repo::list_with_display_names(&state.db, session_id).await?;
  let mut participants: Vec<ParticipantInfo> = participants_rows
    .into_iter()
    .map(|(p, name, color)| ParticipantInfo {
      user_id: p.user_id,
      display_name: name,
      color,
    })
    .collect();

  let session = session_repo::find_by_id(&state.db, session_id)
    .await?
    .ok_or(sqlx::Error::RowNotFound)?;

  let host_in_list = participants.iter().any(|p| p.user_id == session.host_id);
  if !host_in_list {
    let host = user_repo::find_by_id(&state.db, session.host_id)
      .await?
      .ok_or(sqlx::Error::RowNotFound)?;
    participants.push(ParticipantInfo {
      user_id: session.host_id,
      display_name: host.display_name,
      color: host.color,
    });
  }

  let (content, version, language) = state
    .sessions
    .get(&session_id)
    .map(|e| (e.content.clone(), e.version, e.language.clone()))
    .unwrap_or_else(|| (session.content.clone(), 0, session.language.clone()));

  let payload = FullSyncPayload {
    content,
    version,
    language,
    participants,
    messages,
    is_owner,
  };

  let msg = ServerMessage::FullSync(payload);
  if let Some(mut ent) = state.sessions.get_mut(&session_id) {
    broadcast::send_to(&mut ent, user.id, &msg).await;
  }
  Ok(())
}

async fn handle_client_message(
  state: &AppState,
  session_id: Uuid,
  user: &User,
  msg: ClientMessage,
) -> bool {
  match msg {
    ClientMessage::TextChange(delta) => {
      handle_text_change(state, session_id, user, delta).await;
      false
    }
    ClientMessage::CursorMove(pos) => {
      handle_cursor_move(state, session_id, user, pos).await;
      false
    }
    ClientMessage::ChatMessage(body) => {
      handle_chat_message(state, session_id, user, body).await;
      false
    }
    ClientMessage::LanguageChange(lang) => {
      handle_language_change(state, session_id, user, lang).await;
      false
    }
    ClientMessage::Leave => {
      remove_participant(state, session_id, user.id, &user.display_name).await;
      true
    }
  }
}

async fn handle_text_change(
  state: &AppState,
  session_id: Uuid,
  user: &User,
  delta: TextChangeDelta,
) {
  let Some(mut ent) = state.sessions.get_mut(&session_id) else {
    return;
  };

  if delta.range.start_line == 0
    || delta.range.start_column == 0
    || delta.range.end_line == 0
    || delta.range.end_column == 0
  {
    let err = ws_err("VALIDATION", "Invalid range");
    broadcast::send_to(&mut ent, user.id, &err).await;
    return;
  }

  let pending = ent.event_buffer.lock().unwrap().len();
  if ent.published_event_count.saturating_add(pending as i32) >= EVENT_CAP {
    drop(ent);
    let _ = session_repo::set_ended(&state.db, session_id).await;
    broadcast_session_ended(state, session_id, SessionEndReason::EventCapReached).await;
    return;
  }

  if delta.version != ent.version {
    let err = ws_err("VERSION_MISMATCH", "Editor version out of sync");
    broadcast::send_to(&mut ent, user.id, &err).await;
    return;
  }

  if !ent.is_editable_by(user.id) {
    let err = ws_err("READ_ONLY", "This session is read-only");
    broadcast::send_to(&mut ent, user.id, &err).await;
    return;
  }

  let created_at = ent.session_created_at;
  if apply_text_delta(&mut ent.content, &delta).is_err() {
    let err = ws_err("VALIDATION", "Invalid edit range");
    broadcast::send_to(&mut ent, user.id, &err).await;
    return;
  }

  ent.version += 1;
  ent.last_event_at = Instant::now();

  let offset_ms = (OffsetDateTime::now_utc() - created_at).whole_milliseconds() as i64;
  let payload = serde_json::to_value(&delta).unwrap_or_else(|_| json!({}));
  ent.event_buffer.lock().unwrap().push(BufferedEvent {
    payload,
    actor_user_id: user.id,
    offset_ms,
    created_at: OffsetDateTime::now_utc(),
  });

  let new_version = ent.version;
  let out = ServerMessage::TextChange(TextChangePayload {
    delta: delta.clone(),
    version: new_version,
    user_id: user.id,
    display_name: user.display_name.clone(),
  });
  broadcast::broadcast_except(&mut ent, user.id, &out).await;
}

async fn handle_cursor_move(
  state: &AppState,
  session_id: Uuid,
  user: &User,
  pos: CursorPosition,
) {
  let Some(mut ent) = state.sessions.get_mut(&session_id) else {
    return;
  };
  let msg = ServerMessage::CursorMove(CursorPayload {
    line: pos.line,
    column: pos.column,
    user_id: user.id,
    display_name: user.display_name.clone(),
    color: user.color.clone(),
  });
  broadcast::broadcast_except(&mut ent, user.id, &msg).await;
}

async fn handle_chat_message(state: &AppState, session_id: Uuid, user: &User, body: ChatContent) {
  let trimmed = body.content.trim();
  if trimmed.is_empty() {
    if let Some(mut ent) = state.sessions.get_mut(&session_id) {
      let err = ws_err("VALIDATION", "Message cannot be empty");
      broadcast::send_to(&mut ent, user.id, &err).await;
    }
    return;
  }
  if trimmed.chars().count() > MAX_CHAT_CHARS {
    if let Some(mut ent) = state.sessions.get_mut(&session_id) {
      let err = ws_err("VALIDATION", "Message too long");
      broadcast::send_to(&mut ent, user.id, &err).await;
    }
    return;
  }

  let row = match message_repo::insert(&state.db, session_id, user.id, trimmed).await {
    Ok(r) => r,
    Err(e) => {
      tracing::error!(error = %e, "chat insert failed");
      return;
    }
  };

  let payload = chat_row_to_payload(&row);
  let msg = ServerMessage::ChatMessage(payload);
  if let Some(mut ent) = state.sessions.get_mut(&session_id) {
    broadcast::broadcast_all(&mut ent, &msg).await;
  }
}

async fn handle_language_change(
  state: &AppState,
  session_id: Uuid,
  user: &User,
  lang: LanguagePayload,
) {
  let updated = match session_repo::update_language(&state.db, session_id, lang.language).await {
    Ok(s) => s,
    Err(e) => {
      tracing::error!(error = %e, "language update failed");
      return;
    }
  };

  let Some(mut ent) = state.sessions.get_mut(&session_id) else {
    return;
  };
  ent.language = updated.language.clone();

  let msg = ServerMessage::LanguageChange(LanguageChangePayload {
    language: updated.language,
    user_id: user.id,
    display_name: user.display_name.clone(),
  });
  broadcast::broadcast_all(&mut ent, &msg).await;
}

async fn handle_disconnect(
  state: &AppState,
  session_id: Uuid,
  user_id: Uuid,
  display_name: &str,
) {
  remove_participant(state, session_id, user_id, display_name).await;
}

pub async fn remove_participant(
  state: &AppState,
  session_id: Uuid,
  user_id: Uuid,
  display_name: &str,
) {
  let _ = participant_repo::mark_left(&state.db, session_id, user_id).await;

  let remaining = {
    let mut ent = match state.sessions.get_mut(&session_id) {
      Some(e) => e,
      None => return,
    };
    ent.participants.retain(|p| p.user_id != user_id);
    let left = ParticipantLeavePayload {
      user_id,
      display_name: display_name.to_string(),
    };
    let msg = ServerMessage::ParticipantLeave(left);
    let count = ent.participants.len();
    broadcast::broadcast_all(&mut ent, &msg).await;
    count
  };

  tracing::info!(
    session_id = %session_id,
    user_id = %user_id,
    participants = remaining,
    "websocket participant disconnected"
  );

  if remaining == 0 {
    let state_clone = state.clone();
    tokio::spawn(async move {
      tokio::time::sleep(IDLE_TIMEOUT).await;
      let Some(ent) = state_clone.sessions.get(&session_id) else {
        return;
      };
      if !ent.participants.is_empty() {
        return;
      }
      drop(ent);
      let s = match session_repo::find_by_id(&state_clone.db, session_id).await {
        Ok(Some(s)) => s,
        _ => return,
      };
      if s.status != SessionStatus::Active {
        return;
      }
      if session_repo::set_ended(&state_clone.db, session_id).await.is_err() {
        return;
      }
      broadcast_session_ended(&state_clone, session_id, SessionEndReason::IdleTimeout).await;
      tracing::info!(session_id = %session_id, "session ended: idle timeout");
    });
  }
}

fn spawn_flush_loop(state: AppState, session_id: Uuid) {
  tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    loop {
      interval.tick().await;
      if !state.sessions.contains_key(&session_id) {
        break;
      }
      let drained: Vec<BufferedEvent> = {
        let Some(ent) = state.sessions.get(&session_id) else {
          break;
        };
        let mut g = ent.event_buffer.lock().unwrap();
        g.drain()
      };
      if drained.is_empty() {
        continue;
      }
      let n = drained.len() as i32;
      let content = state
        .sessions
        .get(&session_id)
        .map(|e| e.content.clone())
        .unwrap_or_default();
      if let Err(e) =
        session_repo::apply_content_and_increment_events(&state.db, session_id, &content, n).await
      {
        tracing::warn!(error = %e, session_id = %session_id, "flush failed");
        continue;
      }
      if let Some(mut ent) = state.sessions.get_mut(&session_id) {
        ent.published_event_count += n;
      }
    }
  });
}

/// Update in-memory visibility after REST `PATCH .../visibility` (called from session handler).
pub fn sync_active_session_visibility(
  state: &AppState,
  session_id: Uuid,
  visibility: crate::models::SessionVisibility,
) {
  if let Some(mut ent) = state.sessions.get_mut(&session_id) {
    ent.visibility = visibility;
  }
}
