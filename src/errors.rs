// errors.rs
use axum::{
  Json,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  // ── Authentication ────────────────────────────────────────────────────────
  /// X-User-Id header is missing entirely
  #[error("missing user id header")]
  MissingUserId,

  /// X-User-Id header is present but not a valid UUID
  #[error("invalid user id format")]
  InvalidUserId,

  /// UUID is valid but does not exist in the users table
  #[error("user not found")]
  UserNotFound,

  // ── Authorization ─────────────────────────────────────────────────────────
  /// User is authenticated but does not own the resource they are mutating
  #[error("you do not have permission to perform this action")]
  Forbidden,

  // ── Validation ────────────────────────────────────────────────────────────
  /// Request body failed validator derive checks (name too long, empty, etc.)
  #[error("validation error: {0}")]
  Validation(String),

  // ── Not found ─────────────────────────────────────────────────────────────
  /// Session ID (UUID or short_id) does not exist in the database
  #[error("session not found")]
  SessionNotFound,

  // ── Session state ─────────────────────────────────────────────────────────
  /// Attempt to mutate, join, or connect to an ended session
  #[error("this session has ended")]
  SessionEnded,

  /// Attempt to end a session that is already ended
  #[error("session is already ended")]
  SessionAlreadyEnded,

  /// text_change received but session has hit the 100,000 event cap
  #[error("session event limit reached")]
  SessionEventCapReached,

  // ── Visibility / access ───────────────────────────────────────────────────
  /// Non-owner attempting to access a private session
  #[error("this session is private")]
  SessionPrivate,

  /// Non-owner attempting to edit a view_only session
  #[error("this session is read-only")]
  SessionReadOnly,

  // ── WebSocket ─────────────────────────────────────────────────────────────
  /// WebSocket message could not be deserialized into a known message type
  #[error("invalid websocket message")]
  InvalidWsMessage,

  /// WebSocket upgrade attempted on an ended session
  /// Separate from SessionEnded so we can return 410 specifically on upgrade
  #[error("cannot connect to an ended session")]
  WsSessionEnded,

  // ── External services ─────────────────────────────────────────────────────
  /// Judge0 request failed — network error, timeout, or bad response
  #[error("code execution service unavailable")]
  ExecutionServiceUnavailable,

  /// Judge0 returned a response but it could not be parsed
  #[error("code execution returned an unexpected response")]
  ExecutionResponseInvalid,

  // ── Database ──────────────────────────────────────────────────────────────
  /// A unique constraint was violated (e.g. duplicate short_id — extremely rare)
  #[error("a conflicting record already exists")]
  Conflict,

  /// sqlx error that does not map to a specific variant — becomes a 500
  #[error("database error: {0}")]
  Database(#[from] sqlx::Error),

  // ── Catch-all internal ────────────────────────────────────────────────────
  /// Any unexpected error — wraps anyhow for context chain
  #[error("internal server error")]
  Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let (status, code, message) = match &self {
      // 400
      AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.as_str()),
      AppError::InvalidWsMessage => (
        StatusCode::BAD_REQUEST,
        "INVALID_WS_MESSAGE",
        "Invalid websocket message format",
      ),

      // 401
      AppError::MissingUserId => (
        StatusCode::UNAUTHORIZED,
        "MISSING_USER_ID",
        "X-User-Id header is required",
      ),
      AppError::InvalidUserId => (
        StatusCode::UNAUTHORIZED,
        "INVALID_USER_ID",
        "X-User-Id must be a valid UUID",
      ),
      AppError::UserNotFound => (
        StatusCode::UNAUTHORIZED,
        "USER_NOT_FOUND",
        "No user found for the provided ID",
      ),

      // 403
      AppError::Forbidden => (
        StatusCode::FORBIDDEN,
        "FORBIDDEN",
        "You do not have permission to perform this action",
      ),
      AppError::SessionPrivate => (
        StatusCode::FORBIDDEN,
        "SESSION_PRIVATE",
        "This session is private",
      ),
      AppError::SessionReadOnly => (
        StatusCode::FORBIDDEN,
        "SESSION_READ_ONLY",
        "This session is read-only",
      ),

      // 404
      AppError::SessionNotFound => (
        StatusCode::NOT_FOUND,
        "SESSION_NOT_FOUND",
        "Session not found",
      ),

      // 409
      AppError::Conflict => (
        StatusCode::CONFLICT,
        "CONFLICT",
        "A conflicting record already exists",
      ),
      AppError::SessionAlreadyEnded => (
        StatusCode::CONFLICT,
        "SESSION_ALREADY_ENDED",
        "This session has already ended",
      ),

      // 410
      AppError::SessionEnded => (StatusCode::GONE, "SESSION_ENDED", "This session has ended"),
      AppError::WsSessionEnded => (
        StatusCode::GONE,
        "SESSION_ENDED",
        "Cannot connect to an ended session",
      ),

      // 422
      AppError::SessionEventCapReached => (
        StatusCode::UNPROCESSABLE_ENTITY,
        "EVENT_CAP_REACHED",
        "This session has reached the maximum event limit",
      ),

      // 503
      AppError::ExecutionServiceUnavailable => (
        StatusCode::SERVICE_UNAVAILABLE,
        "EXECUTION_UNAVAILABLE",
        "Code execution service is currently unavailable",
      ),
      AppError::ExecutionResponseInvalid => (
        StatusCode::SERVICE_UNAVAILABLE,
        "EXECUTION_INVALID_RESPONSE",
        "Code execution returned an unexpected response",
      ),

      // 500
      AppError::Database(e) => {
        tracing::error!(error = %e, "database error");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "DATABASE_ERROR",
          "An internal error occurred",
        )
      }
      AppError::Internal(e) => {
        tracing::error!(error = %e, "internal error");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "INTERNAL_ERROR",
          "An internal error occurred",
        )
      }
    };

    let body = Json(json!({
        "error": {
            "code":    code,
            "message": message,
        }
    }));

    (status, body).into_response()
  }
}

// ── Layer error conversions ────────────────────────────────────────────────────
// These let the ? operator convert lower-level errors into AppError automatically

#[derive(Debug, Error)]
pub enum RepoError {
  #[error("not found")]
  NotFound,

  #[error("conflict")]
  Conflict,

  #[error("database error: {0}")]
  Database(#[from] sqlx::Error),
}

impl From<RepoError> for AppError {
  fn from(e: RepoError) -> Self {
    match e {
      RepoError::NotFound => AppError::SessionNotFound,
      RepoError::Conflict => AppError::Conflict,
      RepoError::Database(e) => AppError::Database(e),
    }
  }
}

#[derive(Debug, Error)]
pub enum ServiceError {
  #[error("forbidden")]
  Forbidden,

  #[error("session ended")]
  SessionEnded,

  #[error("session already ended")]
  SessionAlreadyEnded,

  #[error("session private")]
  SessionPrivate,

  #[error("session read only")]
  SessionReadOnly,

  #[error("event cap reached")]
  EventCapReached,

  #[error("repo error: {0}")]
  Repo(#[from] RepoError),
}

impl From<ServiceError> for AppError {
  fn from(e: ServiceError) -> Self {
    match e {
      ServiceError::Forbidden => AppError::Forbidden,
      ServiceError::SessionEnded => AppError::SessionEnded,
      ServiceError::SessionAlreadyEnded => AppError::SessionAlreadyEnded,
      ServiceError::SessionPrivate => AppError::SessionPrivate,
      ServiceError::SessionReadOnly => AppError::SessionReadOnly,
      ServiceError::EventCapReached => AppError::SessionEventCapReached,
      ServiceError::Repo(e) => AppError::from(e),
    }
  }
}
