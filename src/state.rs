// src/state.rs
use std::time::Instant;

use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures_util::stream::SplitSink;
use sqlx::PgPool;
use std::sync::Arc;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::config::Config;
use crate::models::{SessionLanguage, SessionVisibility};
use crate::ws::event_buffer::EventBuffer;

/// Represents a live WebSocket session in memory.
/// Populated when the first participant connects,
/// cleared when the session ends or all participants disconnect.
pub struct ActiveSession {
  pub session_id: Uuid,
  pub host_id: Uuid,
  /// For buffered event `offset_ms` (vs session creation).
  pub session_created_at: OffsetDateTime,
  pub content: String,
  pub version: u64,
  pub language: SessionLanguage,
  pub visibility: SessionVisibility,
  pub participants: Vec<ActiveParticipant>,
  pub last_event_at: Instant,
  /// Mirrors DB `event_count` after the last successful flush.
  pub published_event_count: i32,
  pub event_buffer: std::sync::Mutex<EventBuffer>,
}

pub struct ActiveParticipant {
  pub user_id: Uuid,
  pub display_name: String,
  pub color: String,
  pub sender: SplitSink<WebSocket, Message>,
}

impl ActiveSession {
  /// Editability uses live `visibility` + `host_id` (kept in sync when visibility changes over REST).
  pub fn is_editable_by(&self, user_id: Uuid) -> bool {
    match self.visibility {
      SessionVisibility::Edit => true,
      SessionVisibility::ViewOnly | SessionVisibility::Private => self.host_id == user_id,
    }
  }
}

#[derive(Clone)]
pub struct AppState {
  pub db: PgPool,
  pub config: Arc<Config>,
  pub sessions: Arc<DashMap<Uuid, ActiveSession>>,
}

impl AppState {
  pub fn new(db: PgPool, config: Config) -> Self {
    Self {
      db,
      config: Arc::new(config),
      sessions: Arc::new(DashMap::new()),
    }
  }
}
