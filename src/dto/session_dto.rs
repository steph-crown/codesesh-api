// src/dto/session_dto.rs
use crate::models::{
  Session, SessionLanguage, SessionParticipant, SessionStatus, SessionVisibility, User,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

// ─── Request DTOs ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSessionRequest {
  #[validate(length(
    min = 1,
    max = 255,
    message = "name must be between 1 and 255 characters"
  ))]
  pub name: String,

  pub language: SessionLanguage,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSessionNameRequest {
  #[validate(length(
    min = 1,
    max = 255,
    message = "name must be between 1 and 255 characters"
  ))]
  pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSessionVisibilityRequest {
  pub visibility: SessionVisibility,
}

#[derive(Debug, Deserialize)]
pub struct GetSessionsQuery {
  pub search: Option<String>,
  pub created_by_me: Option<bool>,
  pub shared_with_me: Option<bool>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct GetMessagesQuery {
  pub limit: Option<i64>,
  pub before: Option<Uuid>,
}

// ─── Response DTOs ────────────────────────────────────────────────────────────

/// Returned when listing sessions — summary only, no content field
/// content can be large, no need to send it in a list response
#[derive(Debug, Serialize)]
pub struct SessionSummaryResponse {
  pub id: Uuid,
  pub short_id: String,
  pub name: String,
  pub language: SessionLanguage,
  pub visibility: SessionVisibility,
  pub status: SessionStatus,
  pub event_count: i32,
  pub is_owner: bool,
  pub last_activity_at: OffsetDateTime,
  pub created_at: OffsetDateTime,
}

/// Returned when fetching a single session — includes content
#[derive(Debug, Serialize)]
pub struct SessionDetailResponse {
  pub id: Uuid,
  pub short_id: String,
  pub name: String,
  pub language: SessionLanguage,
  pub visibility: SessionVisibility,
  pub status: SessionStatus,
  pub content: String,
  pub event_count: i32,
  pub is_owner: bool,
  pub last_activity_at: OffsetDateTime,
  pub created_at: OffsetDateTime,
  pub updated_at: OffsetDateTime,
}

/// Generic paginated list wrapper — reusable across any list endpoint
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
  pub data: Vec<T>,
  pub total: i64,
  pub page: i64,
  pub limit: i64,
  pub has_more: bool,
}

// ─── Constructors ─────────────────────────────────────────────────────────────

impl SessionSummaryResponse {
  pub fn from_session(session: Session, requesting_user_id: Uuid) -> Self {
    let is_owner = session.is_owned_by(requesting_user_id);
    Self {
      id: session.id,
      short_id: session.short_id,
      name: session.name,
      language: session.language,
      visibility: session.visibility,
      status: session.status,
      event_count: session.event_count,
      is_owner,
      last_activity_at: session.last_activity_at,
      created_at: session.created_at,
    }
  }
}

impl SessionDetailResponse {
  pub fn from_session(session: Session, requesting_user_id: Uuid) -> Self {
    let is_owner = session.is_owned_by(requesting_user_id);
    Self {
      id: session.id,
      short_id: session.short_id,
      name: session.name,
      language: session.language,
      visibility: session.visibility,
      status: session.status,
      content: session.content,
      event_count: session.event_count,
      is_owner,
      last_activity_at: session.last_activity_at,
      created_at: session.created_at,
      updated_at: session.updated_at,
    }
  }
}

impl<T: Serialize> PaginatedResponse<T> {
  pub fn new(data: Vec<T>, total: i64, page: i64, limit: i64) -> Self {
    Self {
      has_more: (page * limit) < total,
      data,
      total,
      page,
      limit,
    }
  }
}
