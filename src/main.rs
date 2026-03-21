use axum::{Router, response::Html, routing::get};
use config::Config;
use state::AppState;

pub mod config;
pub mod errors;

mod db;
mod dto;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;

mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let _ = dotenvy::dotenv();

  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();

  let config = Config::load()?;
  let db_pool = db::create_pool(&config)
    .await
    .map_err(|e| anyhow::anyhow!("Could not connect to database. Failed with error: {e}"))?;

  sqlx::migrate!("./migrations").run(&db_pool).await?;

  let state = AppState::new(db_pool, config.clone());

  let app = Router::new().route("/", get(root));

  let addr = format!("{}:{}", config.host, config.port);
  let listener = tokio::net::TcpListener::bind(&addr).await?;
  tracing::info!("listening on {}", addr);
  axum::serve(listener, app).await?;

  Ok(())
}

async fn root() -> &'static str {
  "Welcome to the codesesh, babyyyy!"
}
