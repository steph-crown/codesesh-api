use axum::{
  Json, Router,
  routing::{Route, get},
};
use serde_json::json;

use crate::state::AppState;

pub fn app_router(state: AppState) -> Router {
  Router::new().nest("/api", api_routes(state))
}

async fn root() -> &'static str {
  "Welcome to the codesesh, babyyyy!"
}

fn api_routes(state: AppState) -> Router {
  Router::new().route("/", get(root)).with_state(state)
}
