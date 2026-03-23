//! Rate limit behavior (tower_governor on `/api` only).

use axum::{
  body::Body,
  http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use codesesh_api::{config::Config, db, routes, state::AppState};

// Fixed client IP so the governor bucket is deterministic for this test.
const TEST_IP: &str = "203.0.113.50";

// Unique body per request so POST /api/users always succeeds.
fn make_user_json() -> String {
  let id = Uuid::new_v4();
  format!(
    r##"{{"display_name":"RateLimitTest-{id}","color":"#336699"}}"##,
    id = id
  )
}

#[tokio::test]
async fn api_returns_429_when_burst_exceeded() {
  dotenvy::dotenv().ok();
  let mut config = Config::load()
    .expect("set DATABASE_URL, FRONTEND_URL, JUDGE0_URL for integration tests (see .env.example)");
  config.rate_limit_per_second = 1;
  config.rate_limit_burst_size = 2;

  let pool = db::create_pool(&config)
    .await
    .expect("connect DATABASE_URL");
  sqlx::migrate!("./migrations")
    .run(&pool)
    .await
    .expect("run migrations");

  let app = routes::app_router(AppState::new(pool, config));

  for i in 0..3 {
    let req = Request::builder()
      .method("POST")
      .uri("/api/users")
      .header("content-type", "application/json")
      .header("x-forwarded-for", TEST_IP)
      .body(Body::from(make_user_json()))
      .expect("request");

    let res = app.clone().oneshot(req).await.expect("response");
    let status = res.status();

    if i < 2 {
      assert_eq!(
        status,
        StatusCode::CREATED,
        "request {i} should succeed before burst is exhausted"
      );
    } else {
      assert_eq!(
        status,
        StatusCode::TOO_MANY_REQUESTS,
        "request {i} should be rate limited"
      );
      assert!(
        res.headers().get("retry-after").is_some(),
        "expected Retry-After header on 429"
      );
    }
  }
}

#[tokio::test]
async fn health_is_not_rate_limited() {
  dotenvy::dotenv().ok();
  let mut config = Config::load()
    .expect("set DATABASE_URL, FRONTEND_URL, JUDGE0_URL for integration tests (see .env.example)");
  config.rate_limit_per_second = 1;
  config.rate_limit_burst_size = 1;

  let pool = db::create_pool(&config)
    .await
    .expect("connect DATABASE_URL");
  sqlx::migrate!("./migrations")
    .run(&pool)
    .await
    .expect("run migrations");

  let app = routes::app_router(AppState::new(pool, config));

  for _ in 0..5 {
    let req = Request::builder()
      .method("GET")
      .uri("/health")
      .body(Body::empty())
      .expect("request");
    let res = app.clone().oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.expect("body").to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(v["success"], json!(true));
  }
}
