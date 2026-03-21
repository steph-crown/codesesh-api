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
    let raw = parts
      .headers
      .get(&X_USER_ID)
      .ok_or(AppError::MissingUserId)?
      .to_str()
      .map_err(|_| AppError::InvalidUserId)?;

    let id = Uuid::parse_str(raw.trim()).map_err(|_| AppError::InvalidUserId)?;

    let user = user_repo::find_by_id(&state.db, id)
      .await
      .map_err(AppError::from)?
      .ok_or(AppError::UserNotFound)?;

    Ok(AuthUser(user))
  }
}
