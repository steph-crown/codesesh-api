//! Custom extractors that map Axum rejections to [`AppError`](crate::errors::AppError).

use std::ops::Deref;

use axum::extract::{FromRequest, Json, Request};
use serde::de::DeserializeOwned;

use crate::errors::AppError;

/// Like [`axum::Json`], but JSON failures become [`AppError`] with the standard `{ error: { code, message } }` body.
#[derive(Debug, Clone, Copy, Default)]
pub struct AppJson<T>(pub T);

impl<S, T> FromRequest<S> for AppJson<T>
where
  T: DeserializeOwned + Send + 'static,
  S: Send + Sync,
{
  type Rejection = AppError;

  async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
    match Json::<T>::from_request(req, state).await {
      Ok(Json(inner)) => Ok(AppJson(inner)),
      Err(e) => Err(AppError::from(e)),
    }
  }
}

impl<T> Deref for AppJson<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
