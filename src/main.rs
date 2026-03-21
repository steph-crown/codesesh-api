use config::Config;

pub mod config;

fn main() -> anyhow::Result<()> {
  let _ = dotenvy::dotenv();
  let config = Config::load()?;

  Ok(())
}
