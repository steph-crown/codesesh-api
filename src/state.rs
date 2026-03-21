// src/state.rs
use crate::config::Config;
use dashmap::DashMap;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Represents a live WebSocket session in memory.
/// Populated when the first participant connects,
/// cleared when the session ends or all participants disconnect.
pub struct ActiveSession {
  pub content: String,
  pub version: u64,
  pub participants: Vec<ActiveParticipant>,
}

pub struct ActiveParticipant {
  pub user_id: Uuid,
  pub display_name: String,
  pub color: String,
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
