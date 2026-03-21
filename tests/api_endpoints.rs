//! HTTP integration tests for every public route.
//!
//! **Setup:** `DATABASE_URL`, `FRONTEND_URL`, and `JUDGE0_URL` must be set (e.g. `dotenvy` / `.env`).
//! Start Postgres (e.g. `docker compose up -d`) before `cargo test --test api_endpoints`.
//!
//! **TDD:** Tests in `contract_pending` are `#[ignore]` until auth, validation, and persistence match
//! `.cursor/project.mdc`. Run them with `cargo test --test api_endpoints -- --ignored`.
//! **Smoke** tests should pass against the current stub handlers.

use std::sync::OnceLock;

use axum::{
  body::Body,
  http::{header, Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tokio::sync::{Mutex, OnceCell};
use tower::ServiceExt;

use codesesh_api::{config::Config, db, routes, state::AppState};

// ─── shared router (one pool + migrations per test process) ─────────────────

static ROUTER: OnceCell<axum::Router> = OnceCell::const_new();

/// Serialize DB-backed requests so the shared pool is not exhausted under parallel tests.
static CALL_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

async fn router() -> axum::Router {
  ROUTER
    .get_or_init(|| async {
      dotenvy::dotenv().ok();
      let config = Config::load().expect(
        "set DATABASE_URL, FRONTEND_URL, JUDGE0_URL for integration tests (see .env.example)",
      );
      let pool = db::create_pool(&config)
        .await
        .expect("connect DATABASE_URL");
      sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations");
      let state = AppState::new(pool, config.clone());
      routes::app_router(state)
    })
    .await
    .clone()
}

async fn call(
  method: Method,
  uri: &str,
  body: Option<&str>,
  extra_headers: &[(&str, &str)],
) -> axum::response::Response {
  let _lock = CALL_LOCK
    .get_or_init(|| Mutex::new(()))
    .lock()
    .await;

  let mut builder = Request::builder().method(method).uri(uri);
  if body.is_some() {
    builder = builder.header(header::CONTENT_TYPE, "application/json");
  }
  for (k, v) in extra_headers {
    builder = builder.header(*k, *v);
  }
  let body = if let Some(b) = body {
    Body::from(b.to_owned())
  } else {
    Body::empty()
  };
  let req = builder.body(body).expect("request");
  router()
    .await
    .oneshot(req)
    .await
    .expect("response")
}

async fn body_json(res: axum::response::Response) -> Value {
  let bytes = res
    .into_body()
    .collect()
    .await
    .expect("body")
    .to_bytes();
  serde_json::from_slice(&bytes).expect("json body")
}

/// Successful API envelope: `{ "success": true, "data": ... }` (see `response::ApiResponse`).
fn unwrap_data(v: &Value) -> &Value {
  assert_eq!(
    v.get("success"),
    Some(&json!(true)),
    "expected success envelope, got {v}"
  );
  v.get("data").expect("data field")
}

/// Error envelope: `{ "error": { "code", "message" } }` (see `errors::AppError`).
fn unwrap_error(v: &Value) -> (&str, &str) {
  let err = v.get("error").expect("error object");
  let code = err
    .get("code")
    .and_then(|c| c.as_str())
    .expect("error.code");
  let message = err
    .get("message")
    .and_then(|m| m.as_str())
    .expect("error.message");
  (code, message)
}

fn session_path(session_id: &str, suffix: &str) -> String {
  format!("/api/sessions/{session_id}{suffix}")
}

async fn create_user() -> String {
  let res = call(
    Method::POST,
    "/api/users",
    Some(r#"{"display_name":"ApiTestUser"}"#),
    &[],
  )
  .await;
  assert_eq!(res.status(), StatusCode::CREATED);
  let v = body_json(res).await;
  unwrap_data(&v)["id"]
    .as_str()
    .expect("user id")
    .to_string()
}

async fn create_session(user_id: &str) -> String {
  let res = call(
    Method::POST,
    "/api/sessions",
    Some(r#"{"name":"Pairing","language":"typescript"}"#),
    &[("X-User-Id", user_id)],
  )
  .await;
  assert_eq!(res.status(), StatusCode::CREATED);
  let v = body_json(res).await;
  unwrap_data(&v)["id"]
    .as_str()
    .expect("session id")
    .to_string()
}

// ═══════════════════════════════════════════════════════════════════════════
// Smoke — should pass with stub handlers (happy path / routing)
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn health_get_returns_ok_json() {
  let res = call(Method::GET, "/health", None, &[]).await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert_eq!(unwrap_data(&v)["status"], json!("ok"));
}

#[tokio::test]
async fn post_users_valid_body_returns_201_and_shape() {
  let res = call(
    Method::POST,
    "/api/users",
    Some(r#"{"display_name":"Ada"}"#),
    &[],
  )
  .await;
  assert_eq!(res.status(), StatusCode::CREATED);
  let v = body_json(res).await;
  let d = unwrap_data(&v);
  assert!(d.get("id").is_some());
  assert_eq!(d["display_name"], json!("Ada"));
}

#[tokio::test]
async fn get_sessions_returns_paginated_shape() {
  let uid = create_user().await;
  let res = call(
    Method::GET,
    "/api/sessions",
    None,
    &[("X-User-Id", uid.as_str())],
  )
  .await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  let d = unwrap_data(&v);
  for key in ["data", "total", "page", "limit", "has_more"] {
    assert!(
      d.get(key).is_some(),
      "missing key {key} in {d}"
    );
  }
}

#[tokio::test]
async fn get_sessions_accepts_query_params() {
  let uid = create_user().await;
  let uri = "/api/sessions?search=foo&created_by_me=true&shared_with_me=false&page=2&limit=10";
  let res = call(Method::GET, uri, None, &[("X-User-Id", uid.as_str())])
    .await;
  assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn post_sessions_valid_body_returns_detail_shape() {
  let uid = create_user().await;
  let res = call(
    Method::POST,
    "/api/sessions",
    Some(r#"{"name":"Pairing","language":"typescript"}"#),
    &[("X-User-Id", uid.as_str())],
  )
  .await;
  assert_eq!(res.status(), StatusCode::CREATED);
  let v = body_json(res).await;
  let d = unwrap_data(&v);
  for key in [
    "id",
    "short_id",
    "name",
    "language",
    "visibility",
    "status",
    "content",
    "event_count",
    "is_owner",
    "last_activity_at",
    "created_at",
    "updated_at",
  ] {
    assert!(d.get(key).is_some(), "missing {key}");
  }
}

#[tokio::test]
async fn get_session_by_id_returns_detail_shape() {
  let uid = create_user().await;
  let sid = create_session(&uid).await;
  let res = call(
    Method::GET,
    &session_path(&sid, ""),
    None,
    &[("X-User-Id", uid.as_str())],
  )
  .await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert!(
    unwrap_data(&v).get("id").is_some(),
    "detail response should include id (stub may use nil UUID)"
  );
}

#[tokio::test]
async fn post_join_returns_participant_shape() {
  let host = create_user().await;
  let guest = create_user().await;
  let sid = create_session(&host).await;
  let res = call(
    Method::POST,
    &session_path(&sid, "/join"),
    None,
    &[("X-User-Id", guest.as_str())],
  )
  .await;
  assert_eq!(res.status(), StatusCode::CREATED);
  let v = body_json(res).await;
  let d = unwrap_data(&v);
  for key in ["user_id", "display_name", "joined_at", "is_active"] {
    assert!(d.get(key).is_some(), "missing {key}");
  }
}

#[tokio::test]
async fn get_participants_returns_array() {
  let uid = create_user().await;
  let sid = create_session(&uid).await;
  let res = call(
    Method::GET,
    &session_path(&sid, "/participants"),
    None,
    &[("X-User-Id", uid.as_str())],
  )
  .await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert!(unwrap_data(&v).is_array());
}

#[tokio::test]
async fn patch_session_name_returns_detail_shape() {
  let uid = create_user().await;
  let sid = create_session(&uid).await;
  let res = call(
    Method::PATCH,
    &session_path(&sid, "/name"),
    Some(r#"{"name":"Renamed"}"#),
    &[("X-User-Id", uid.as_str())],
  )
  .await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert!(unwrap_data(&v).get("name").is_some());
}

#[tokio::test]
async fn patch_session_visibility_returns_detail_shape() {
  let uid = create_user().await;
  let sid = create_session(&uid).await;
  let res = call(
    Method::PATCH,
    &session_path(&sid, "/visibility"),
    Some(r#"{"visibility":"private"}"#),
    &[("X-User-Id", uid.as_str())],
  )
  .await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert!(unwrap_data(&v).get("visibility").is_some());
}

#[tokio::test]
async fn patch_session_end_returns_detail_shape() {
  let uid = create_user().await;
  let sid = create_session(&uid).await;
  let res = call(
    Method::PATCH,
    &session_path(&sid, "/end"),
    None,
    &[("X-User-Id", uid.as_str())],
  )
  .await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert_eq!(unwrap_data(&v)["status"], json!("ended"));
}

#[tokio::test]
async fn get_messages_returns_history_shape() {
  let uid = create_user().await;
  let sid = create_session(&uid).await;
  let res = call(
    Method::GET,
    &session_path(&sid, "/messages"),
    None,
    &[("X-User-Id", uid.as_str())],
  )
  .await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  let d = unwrap_data(&v);
  assert!(d.get("messages").is_some());
  assert!(d.get("has_more").is_some());
}

#[tokio::test]
async fn get_messages_accepts_query_params() {
  let uid = create_user().await;
  let sid = create_session(&uid).await;
  let uri = format!(
    "{}/messages?limit=25&before=00000000-0000-0000-0000-000000000099",
    session_path(&sid, "")
  );
  let res = call(
    Method::GET,
    &uri,
    None,
    &[("X-User-Id", uid.as_str())],
  )
  .await;
  assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn post_execute_returns_json_object() {
  let uid = create_user().await;
  let sid = create_session(&uid).await;
  let res = call(
    Method::POST,
    &session_path(&sid, "/execute"),
    None,
    &[("X-User-Id", uid.as_str())],
  )
  .await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert!(unwrap_data(&v).is_object());
}

#[tokio::test]
async fn get_ws_is_routed() {
  let uid = create_user().await;
  let sid = create_session(&uid).await;
  let res = call(
    Method::GET,
    &session_path(&sid, "/ws"),
    None,
    &[("X-User-Id", uid.as_str())],
  )
  .await;
  let s = res.status();
  assert_ne!(s, StatusCode::NOT_FOUND);
  assert_ne!(s, StatusCode::METHOD_NOT_ALLOWED);
}

// ═══════════════════════════════════════════════════════════════════════════
// Routing & HTTP edge cases (Axum / method behavior)
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn wrong_method_post_health_returns_405() {
  let res = call(Method::POST, "/health", None, &[]).await;
  assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn invalid_uuid_in_session_path_returns_4xx() {
  let res = call(
    Method::GET,
    "/api/sessions/not-a-uuid",
    None,
    &[],
  )
  .await;
  assert!(
    res.status().is_client_error(),
    "expected 4xx for invalid UUID path, got {}",
    res.status()
  );
}

#[tokio::test]
async fn unknown_api_route_returns_404() {
  let res = call(Method::GET, "/api/does-not-exist", None, &[]).await;
  assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn post_users_malformed_json_returns_400_app_error_shape() {
  let res = call(Method::POST, "/api/users", Some("{not json"), &[]).await;
  assert_eq!(res.status(), StatusCode::BAD_REQUEST);
  let v = body_json(res).await;
  let (code, _) = unwrap_error(&v);
  assert_eq!(code, "INVALID_JSON_SYNTAX");
}

#[tokio::test]
async fn post_users_missing_field_returns_422_app_error_shape() {
  let res = call(Method::POST, "/api/users", Some("{}"), &[]).await;
  assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
  let v = body_json(res).await;
  let (code, message) = unwrap_error(&v);
  assert_eq!(code, "INVALID_JSON_BODY");
  assert_eq!(
    message,
    "The request body could not be processed.",
    "client must not receive serde/field path details"
  );
}

// ═══════════════════════════════════════════════════════════════════════════
// Contract (ignored until middleware + services enforce project.mdc)
// Run: cargo test --test api_endpoints -- --ignored
// ═══════════════════════════════════════════════════════════════════════════

mod contract_pending {
  use axum::http::{Method, StatusCode};

  use super::{call, create_session, create_user, session_path};

  #[tokio::test]
  async fn session_mutations_require_x_user_id() {
    let uid = create_user().await;
    let sid = create_session(&uid).await;
    let res = call(
      Method::PATCH,
      &session_path(&sid, "/name"),
      Some(r#"{"name":"x"}"#),
      &[],
    )
    .await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
  }

  #[tokio::test]
  async fn unknown_user_id_returns_401() {
    let uid = create_user().await;
    let sid = create_session(&uid).await;
    let res = call(
      Method::GET,
      &session_path(&sid, ""),
      None,
      &[("X-User-Id", "00000000-0000-0000-0000-00000000c001")],
    )
    .await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
  }

  #[tokio::test]
  async fn get_unknown_session_returns_404() {
    let uid = create_user().await;
    let res = call(
      Method::GET,
      "/api/sessions/11111111-1111-1111-1111-111111111111",
      None,
      &[("X-User-Id", uid.as_str())],
    )
    .await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn mutations_on_ended_session_return_410() {
    let uid = create_user().await;
    let sid = create_session(&uid).await;
    let end = call(
      Method::PATCH,
      &session_path(&sid, "/end"),
      None,
      &[("X-User-Id", uid.as_str())],
    )
    .await;
    assert_eq!(end.status(), StatusCode::OK);
    let res = call(
      Method::PATCH,
      &session_path(&sid, "/name"),
      Some(r#"{"name":"nope"}"#),
      &[("X-User-Id", uid.as_str())],
    )
    .await;
    assert_eq!(res.status(), StatusCode::GONE);
  }

  #[tokio::test]
  #[ignore = "Validate CreateUserRequest: empty display_name -> 400"]
  async fn create_user_empty_display_name_returns_400() {
    let res = call(
      Method::POST,
      "/api/users",
      Some(r#"{"display_name":""}"#),
      &[],
    )
    .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
  }

  #[tokio::test]
  #[ignore = "Private session: non-owner cannot read — needs host + other user + PATCH visibility"]
  async fn get_private_session_forbidden_for_non_owner() {
    let host = create_user().await;
    let other = create_user().await;
    let sid = create_session(&host).await;
    let patch = call(
      Method::PATCH,
      &session_path(&sid, "/visibility"),
      Some(r#"{"visibility":"private"}"#),
      &[("X-User-Id", host.as_str())],
    )
    .await;
    assert_eq!(patch.status(), StatusCode::OK);
    let res = call(
      Method::GET,
      &session_path(&sid, ""),
      None,
      &[("X-User-Id", other.as_str())],
    )
    .await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
  }
}
