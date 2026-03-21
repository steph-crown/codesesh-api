use axum::{
  extract::{
    Path,
    ws::{WebSocket, WebSocketUpgrade},
  },
  response::IntoResponse,
};
/// WebSocket upgrade for `GET /api/sessions/:short_id/ws`.
pub async fn session_websocket(
  ws: WebSocketUpgrade,
  Path(_short_id): Path<String>,
) -> impl IntoResponse {
  ws.on_upgrade(|_socket: WebSocket| async move {})
}
