use axum::{
  extract::{
    Path,
    ws::{WebSocket, WebSocketUpgrade},
  },
  response::IntoResponse,
};
use uuid::Uuid;

/// WebSocket upgrade for `GET /api/sessions/:session_id/ws`.
pub async fn session_websocket(
  ws: WebSocketUpgrade,
  Path(_session_id): Path<Uuid>,
) -> impl IntoResponse {
  ws.on_upgrade(|_socket: WebSocket| async move {})
}
