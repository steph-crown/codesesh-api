//! HTTP rate limiting for `/api` routes (token bucket via [`tower_governor`]).
//!
//! - **Sustained rate** [`Config::rate_limit_per_second`](crate::config::Config): average requests
//!   per second per client key (after burst is consumed).
//! - **Burst** [`Config::rate_limit_burst_size`](crate::config::Config): max bucket size for short spikes.
//!
//! Key: [`SmartIpKeyExtractor`](tower_governor::key_extractor::SmartIpKeyExtractor) — uses
//! `X-Forwarded-For` / `X-Real-IP` / `Forwarded` when present (trusted proxy), else peer IP from
//! [`axum::extract::ConnectInfo`] (see `into_make_service_with_connect_info` in `run`).

use std::sync::Arc;
use std::time::Duration;

use governor::middleware::NoOpMiddleware;
use tower_governor::{
  governor::{GovernorConfig, GovernorConfigBuilder},
  key_extractor::SmartIpKeyExtractor,
  GovernorLayer,
};

use crate::config::Config;

/// Builds the governor layer applied only to nested `/api` routes.
pub fn api_governor_layer(config: &Config) -> GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware> {
  GovernorLayer {
    config: api_governor_config_arc(config),
  }
}

fn api_governor_config_arc(
  config: &Config,
) -> Arc<GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware>> {
  let rate = config.rate_limit_per_second.max(1);
  let period = Duration::from_nanos(1_000_000_000u64 / rate);
  let burst = config.rate_limit_burst_size.max(1);

  let mut builder = GovernorConfigBuilder::const_default();
  builder.period(period);
  builder.burst_size(burst);
  Arc::new(
    builder
      .key_extractor(SmartIpKeyExtractor)
      .finish()
      .expect("rate limit: period and burst must be non-zero"),
  )
}
