use axum::{
  Router,
  routing::{get, patch, post},
};

use crate::handlers::{
  execute::execute_code,
  health::health_check,
  messages::list_messages,
  participants::{create_participant, list_participants},
  sessions::{
    create_session, end_session, get_session, list_sessions, update_session_name,
    update_session_visibility,
  },
  users::create_user,
  ws::session_websocket,
};
use crate::state::AppState;

pub fn app_router(state: AppState) -> Router {
  Router::new()
    .route("/health", get(health_check))
    .nest(
      "/api",
      Router::new()
        .route("/users", post(create_user))
        .route("/sessions", get(list_sessions).post(create_session))
        .route("/sessions/{session_id}", get(get_session))
        .route("/sessions/{session_id}/name", patch(update_session_name))
        .route(
          "/sessions/{session_id}/visibility",
          patch(update_session_visibility),
        )
        .route("/sessions/{session_id}/end", patch(end_session))
        .route(
          "/sessions/{session_id}/participants",
          get(list_participants),
        )
        .route("/sessions/{session_id}/join", post(create_participant))
        .route("/sessions/{session_id}/messages", get(list_messages))
        .route("/sessions/{session_id}/ws", get(session_websocket))
        .route("/sessions/{session_id}/execute", post(execute_code)),
    )
    .with_state(state)
}
