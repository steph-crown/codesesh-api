use config::Config;

pub mod config;
pub mod errors;

mod db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let _ = dotenvy::dotenv();
  let config = Config::load()?;
  let db_pool = db::create_pool(&config)
    .await
    .map_err(|e| anyhow::anyhow!("Could not connect to database. Failed with error: {e}"))?;

  sqlx::migrate!("./migrations").run(&db_pool).await?;

  Ok(())
}
