//! HTTP integration tests for every public route.
//!
//! **Setup:** `DATABASE_URL`, `FRONTEND_URL`, and `JUDGE0_URL` must be set (e.g. `dotenvy` / `.env`).
//! Start Postgres (e.g. `docker compose up -d`) before `cargo test --test api_endpoints`.
//!
//! **TDD:** Tests in `contract_pending` are `#[ignore]` until auth, validation, and persistence match
//! `.cursor/project.mdc`. Run them with `cargo test --test api_endpoints -- --ignored`.
//! **Smoke** tests should pass against the current stub handlers.

use axum::{
  body::Body,
  http::{header, Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tokio::sync::OnceCell;
use tower::ServiceExt;

use codesesh_api::{config::Config, db, routes, state::AppState};

// ─── shared router (one pool + migrations per test process) ─────────────────

static ROUTER: OnceCell<axum::Router> = OnceCell::const_new();

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

fn session_path(suffix: &str) -> String {
  format!("/api/sessions/{SESSION_ID}{suffix}")
}

const SESSION_ID: &str = "00000000-0000-0000-0000-0000000000a1";
const USER_HEADER_ID: &str = "00000000-0000-0000-0000-0000000000b2";

// ═══════════════════════════════════════════════════════════════════════════
// Smoke — should pass with stub handlers (happy path / routing)
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn health_get_returns_ok_json() {
  let res = call(Method::GET, "/health", None, &[]).await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert_eq!(v["status"], json!("ok"));
}

#[tokio::test]
async fn post_users_valid_body_returns_200_and_shape() {
  let res = call(
    Method::POST,
    "/api/users",
    Some(r#"{"display_name":"Ada"}"#),
    &[],
  )
  .await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert!(v.get("id").is_some());
  assert_eq!(v["display_name"], json!("Ada"));
}

#[tokio::test]
async fn get_sessions_returns_paginated_shape() {
  let res = call(Method::GET, "/api/sessions", None, &[]).await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  for key in ["data", "total", "page", "limit", "has_more"] {
    assert!(
      v.get(key).is_some(),
      "missing key {key} in {v}"
    );
  }
}

#[tokio::test]
async fn get_sessions_accepts_query_params() {
  let uri = "/api/sessions?search=foo&created_by_me=true&shared_with_me=false&page=2&limit=10";
  let res = call(Method::GET, uri, None, &[]).await;
  assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn post_sessions_valid_body_returns_detail_shape() {
  let res = call(
    Method::POST,
    "/api/sessions",
    Some(r#"{"name":"Pairing","language":"typescript"}"#),
    &[],
  )
  .await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
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
    assert!(v.get(key).is_some(), "missing {key}");
  }
}

#[tokio::test]
async fn get_session_by_id_returns_detail_shape() {
  let res = call(Method::GET, &session_path(""), None, &[]).await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert!(v.get("id").is_some(), "detail response should include id (stub may use nil UUID)");
}

#[tokio::test]
async fn post_join_returns_participant_shape() {
  let res = call(Method::POST, &session_path("/join"), None, &[]).await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  for key in ["user_id", "display_name", "joined_at", "is_active"] {
    assert!(v.get(key).is_some(), "missing {key}");
  }
}

#[tokio::test]
async fn get_participants_returns_array() {
  let res = call(Method::GET, &session_path("/participants"), None, &[]).await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert!(v.is_array());
}

#[tokio::test]
async fn patch_session_name_returns_detail_shape() {
  let res = call(
    Method::PATCH,
    &session_path("/name"),
    Some(r#"{"name":"Renamed"}"#),
    &[],
  )
  .await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert!(v.get("name").is_some());
}

#[tokio::test]
async fn patch_session_visibility_returns_detail_shape() {
  let res = call(
    Method::PATCH,
    &session_path("/visibility"),
    Some(r#"{"visibility":"private"}"#),
    &[],
  )
  .await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert!(v.get("visibility").is_some());
}

#[tokio::test]
async fn patch_session_end_returns_detail_shape() {
  let res = call(Method::PATCH, &session_path("/end"), None, &[]).await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert_eq!(v["status"], json!("ended"));
}

#[tokio::test]
async fn get_messages_returns_history_shape() {
  let res = call(Method::GET, &session_path("/messages"), None, &[]).await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert!(v.get("messages").is_some());
  assert!(v.get("has_more").is_some());
}

#[tokio::test]
async fn get_messages_accepts_query_params() {
  let uri = format!(
    "{}/messages?limit=25&before=00000000-0000-0000-0000-000000000099",
    session_path("")
  );
  let res = call(Method::GET, &uri, None, &[]).await;
  assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn post_execute_returns_json_object() {
  let res = call(Method::POST, &session_path("/execute"), None, &[]).await;
  assert_eq!(res.status(), StatusCode::OK);
  let v = body_json(res).await;
  assert!(v.is_object());
}

#[tokio::test]
async fn get_ws_is_routed() {
  let res = call(Method::GET, &session_path("/ws"), None, &[]).await;
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
async fn post_users_malformed_json_returns_4xx() {
  let res = call(Method::POST, "/api/users", Some("{not json"), &[]).await;
  assert!(
    res.status().is_client_error(),
    "malformed JSON should be rejected, got {}",
    res.status()
  );
}

// ═══════════════════════════════════════════════════════════════════════════
// Contract (ignored until middleware + services enforce project.mdc)
// Run: cargo test --test api_endpoints -- --ignored
// ═══════════════════════════════════════════════════════════════════════════

mod contract_pending {
  use axum::http::{Method, StatusCode};

  use super::{call, session_path, USER_HEADER_ID};

  #[tokio::test]
  #[ignore = "Phase 3: require X-User-Id on protected routes"]
  async fn session_mutations_require_x_user_id() {
    let res = call(
      Method::PATCH,
      &session_path("/name"),
      Some(r#"{"name":"x"}"#),
      &[],
    )
    .await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
  }

  #[tokio::test]
  #[ignore = "Phase 3: valid X-User-Id must exist in users table"]
  async fn unknown_user_id_returns_401() {
    let res = call(
      Method::GET,
      &session_path(""),
      None,
      &[("X-User-Id", "00000000-0000-0000-0000-00000000c001")],
    )
    .await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
  }

  #[tokio::test]
  #[ignore = "Return 404 when session id does not exist"]
  async fn get_unknown_session_returns_404() {
    let res = call(
      Method::GET,
      "/api/sessions/11111111-1111-1111-1111-111111111111",
      None,
      &[("X-User-Id", USER_HEADER_ID)],
    )
    .await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  #[ignore = "Return 410 when session has ended"]
  async fn mutations_on_ended_session_return_410() {
    let res = call(
      Method::PATCH,
      &session_path("/name"),
      Some(r#"{"name":"nope"}"#),
      &[("X-User-Id", USER_HEADER_ID)],
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
  #[ignore = "Private session: non-owner cannot read"]
  async fn get_private_session_forbidden_for_non_owner() {
    let res = call(
      Method::GET,
      &session_path(""),
      None,
      &[("X-User-Id", USER_HEADER_ID)],
    )
    .await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
  }
}
