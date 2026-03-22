//! CodeSesh API library — same module tree as the binary;

pub mod config;
pub mod db;
pub mod dto;
pub mod errors;
pub mod extractors;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod response;
pub mod routes;
pub mod services;
pub mod state;
pub mod ws;

/// Boot the HTTP server (used by `main`).
pub async fn run() -> anyhow::Result<()> {
  let _ = dotenvy::dotenv();

  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();

  let config = config::Config::load()?;
  tracing::info!(
    host = %config.host,
    port = config.port,
    "configuration loaded",
  );

  let db_pool = db::create_pool(&config)
    .await
    .map_err(|e| anyhow::anyhow!("Could not connect to database. Failed with error: {e}"))?;
  tracing::info!("database connection pool ready");

  sqlx::migrate!("./migrations").run(&db_pool).await?;
  tracing::info!("database migrations applied");

  let state = state::AppState::new(db_pool, config.clone());

  let app = routes::app_router(state);

  let addr = format!("{}:{}", config.host, config.port);
  let listener = tokio::net::TcpListener::bind(&addr).await?;
  tracing::info!("listening on {}", addr);
  axum::serve(listener, app).await?;

  Ok(())
}
