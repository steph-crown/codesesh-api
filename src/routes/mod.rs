use axum::{
  Router,
  routing::{get, post},
};

use crate::{handlers::users::create_user, state::AppState};

pub fn app_router(state: AppState) -> Router {
  Router::new().nest("/api", api_routes(state))
}

async fn root() -> &'static str {
  "Welcome to the codesesh, babyyyy!"
}

fn api_routes(state: AppState) -> Router {
  Router::new()
    .route("/", get(root))
    .route("/users", post(create_user))
    .with_state(state)
}
