//! Authenticated user from `X-User-Id` + database lookup.

use std::ops::Deref;

use axum::extract::FromRequestParts;
use axum::http::header::HeaderName;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::{errors::AppError, models::User, repositories::user_repo, state::AppState};

/// `X-User-Id` (case-insensitive); value must be a UUID of an existing user.
static X_USER_ID: HeaderName = HeaderName::from_static("x-user-id");

/// Resolved [`User`] from the request (see module docs).
#[derive(Debug)]
pub struct AuthUser(pub User);

impl Deref for AuthUser {
  type Target = User;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl FromRequestParts<AppState> for AuthUser {
  type Rejection = AppError;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let raw = match parts.headers.get(&X_USER_ID) {
      Some(v) => v,
      None => {
        tracing::debug!("rejecting request: missing X-User-Id header");
        return Err(AppError::MissingUserId);
      }
    };

    let raw = match raw.to_str() {
      Ok(s) => s,
      Err(_) => {
        tracing::debug!("rejecting request: X-User-Id header is not valid UTF-8");
        return Err(AppError::InvalidUserId);
      }
    };

    let id = match Uuid::parse_str(raw.trim()) {
      Ok(id) => id,
      Err(_) => {
        tracing::debug!("rejecting request: X-User-Id is not a valid UUID");
        return Err(AppError::InvalidUserId);
      }
    };

    let user = match user_repo::find_by_id(&state.db, id).await {
      Ok(Some(user)) => user,
      Ok(None) => {
        tracing::debug!(user_id = %id, "rejecting request: no user for X-User-Id");
        return Err(AppError::UserNotFound);
      }
      Err(e) => {
        tracing::error!(error = %e, user_id = %id, "database error loading user for X-User-Id");
        return Err(AppError::from(e));
      }
    };

    tracing::trace!(user_id = %user.id, "authenticated request");

    Ok(AuthUser(user))
  }
}
