use axum::{
  Json,
  extract::{Path, Query, State},
};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
  dto::{
    CreateSessionRequest, GetSessionsQuery, PaginatedResponse, SessionDetailResponse,
    SessionSummaryResponse, UpdateSessionNameRequest, UpdateSessionVisibilityRequest,
  },
  errors::AppResult,
  models::{SessionLanguage, SessionStatus, SessionVisibility},
  state::AppState,
};

fn stub_session_detail() -> SessionDetailResponse {
  SessionDetailResponse {
    id: Uuid::nil(),
    short_id: "stub".to_string(),
    name: "stub".to_string(),
    language: SessionLanguage::TypeScript,
    visibility: SessionVisibility::Edit,
    status: SessionStatus::Active,
    content: String::new(),
    event_count: 0,
    is_owner: false,
    last_activity_at: OffsetDateTime::UNIX_EPOCH,
    created_at: OffsetDateTime::UNIX_EPOCH,
    updated_at: OffsetDateTime::UNIX_EPOCH,
  }
}

pub async fn create_session(
  State(_state): State<AppState>,
  Json(_payload): Json<CreateSessionRequest>,
) -> AppResult<Json<SessionDetailResponse>> {
  Ok(Json(stub_session_detail()))
}

pub async fn list_sessions(
  State(_state): State<AppState>,
  Query(_query): Query<GetSessionsQuery>,
) -> AppResult<Json<PaginatedResponse<SessionSummaryResponse>>> {
  Ok(Json(PaginatedResponse {
    data: vec![],
    total: 0,
    page: 1,
    limit: 20,
    has_more: false,
  }))
}

pub async fn get_session(
  State(_state): State<AppState>,
  Path(_session_id): Path<Uuid>,
) -> AppResult<Json<SessionDetailResponse>> {
  Ok(Json(stub_session_detail()))
}

pub async fn update_session_name(
  State(_state): State<AppState>,
  Path(_session_id): Path<Uuid>,
  Json(_payload): Json<UpdateSessionNameRequest>,
) -> AppResult<Json<SessionDetailResponse>> {
  Ok(Json(stub_session_detail()))
}

pub async fn update_session_visibility(
  State(_state): State<AppState>,
  Path(_session_id): Path<Uuid>,
  Json(_payload): Json<UpdateSessionVisibilityRequest>,
) -> AppResult<Json<SessionDetailResponse>> {
  Ok(Json(stub_session_detail()))
}

pub async fn end_session(
  State(_state): State<AppState>,
  Path(_session_id): Path<Uuid>,
) -> AppResult<Json<SessionDetailResponse>> {
  let mut body = stub_session_detail();
  body.status = SessionStatus::Ended;
  Ok(Json(body))
}
