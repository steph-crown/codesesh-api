//! WebSocket upgrade for `GET /api/sessions/:short_id/ws?user_id=...`.

use axum::extract::{Path, Query, State, WebSocketUpgrade};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  errors::{AppError, AppResult},
  models::SessionStatus,
  repositories::{participant_repo, session_repo, user_repo},
  services::{session_service, ws_service},
  state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct WsQuery {
  pub user_id: Uuid,
}

pub async fn session_websocket(
  State(state): State<AppState>,
  Path(short_id): Path<String>,
  Query(query): Query<WsQuery>,
  ws: WebSocketUpgrade,
) -> AppResult<impl axum::response::IntoResponse> {
  let user = user_repo::find_by_id(&state.db, query.user_id)
    .await
    .map_err(AppError::from)?
    .ok_or(AppError::UserNotFound)?;

  let session_id = session_service::resolve_session_id(&state.db, &short_id).await?;
  let session = session_repo::find_by_id(&state.db, session_id)
    .await
    .map_err(AppError::from)?
    .ok_or(AppError::SessionNotFound)?;

  if session.status == SessionStatus::Ended {
    return Err(AppError::WsSessionEnded);
  }

  let ok = participant_repo::is_active_participant(&state.db, session_id, user.id)
    .await
    .map_err(AppError::from)?;
  if !ok {
    return Err(AppError::Forbidden);
  }

  let is_owner = session.host_id == user.id;
  let state_clone = state.clone();

  Ok(ws.on_upgrade(move |socket| {
    let state = state_clone;
    async move {
      ws_service::handle_connection(socket, session, user, state, is_owner).await;
    }
  }))
}
