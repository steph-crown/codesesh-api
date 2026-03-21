use axum::{
  Json,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::json;

pub struct ApiResponse<T: Serialize> {
  status: StatusCode,
  data: T,
}

impl<T: Serialize> ApiResponse<T> {
  pub fn ok(data: T) -> Self {
    Self {
      status: StatusCode::OK,
      data,
    }
  }

  pub fn created(data: T) -> Self {
    Self {
      status: StatusCode::CREATED,
      data,
    }
  }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
  fn into_response(self) -> Response {
    let body = Json(json!({
        "success": true,
        "data": self.data,
    }));
    (self.status, body).into_response()
  }
}

/// For endpoints that return no body — just a status code
/// e.g. PATCH /sessions/:id/end → 204 No Content
pub struct ApiNoContent;

impl IntoResponse for ApiNoContent {
  fn into_response(self) -> Response {
    StatusCode::NO_CONTENT.into_response()
  }
}
