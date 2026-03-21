use std::env;

#[derive(Clone)]
pub struct Config {
  // Server
  pub host: String,
  pub port: u16,

  // Database
  pub database_url: String,

  // Frontend (for CORS)
  pub frontend_url: String,

  // Judge0
  pub judge0_url: String,

  // Rate limiting
  pub rate_limit_per_second: u64,
  pub rate_limit_burst_size: u32,
}

impl Config {
  pub fn load() -> Result<Self, anyhow::Error> {
    Ok(Self {
      host: env_or("HOST", "0.0.0.0".to_string()),
      port: env_parse_or("PORT", 8080)?,

      database_url: required("DATABASE_URL")?,

      frontend_url: required("FRONTEND_URL")?,

      judge0_url: required("JUDGE0_URL")?,

      rate_limit_per_second: env_parse_or("RATE_LIMIT_PER_SECOND", 10)?,
      rate_limit_burst_size: env_parse_or("RATE_LIMIT_BURST_SIZE", 30)?,
    })
  }
}

fn required(key: &str) -> Result<String, anyhow::Error> {
  env::var(key).map_err(|_| anyhow::anyhow!("Missing required environment variable: {}", key))
}

fn env_or(key: &str, default: String) -> String {
  env::var(key).unwrap_or(default)
}

fn env_parse_or<T>(key: &str, default: T) -> Result<T, anyhow::Error>
where
  T: std::str::FromStr + std::fmt::Display,
  T::Err: std::fmt::Display,
{
  match env::var(key) {
    Ok(value) => value
      .parse::<T>()
      .map_err(|_| anyhow::anyhow!("Invalid value for {}: {}", key, value)),
    Err(_) => Ok(default),
  }
}
