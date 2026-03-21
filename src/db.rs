use crate::config::Config;
use sqlx::postgres::PgPoolOptions;

pub async fn create_pool(config: &Config) -> Result<sqlx::PgPool, sqlx::Error> {
  PgPoolOptions::new()
    .max_connections(32)
    .acquire_timeout(std::time::Duration::from_secs(3))
    .connect(&config.database_url)
    .await
}
