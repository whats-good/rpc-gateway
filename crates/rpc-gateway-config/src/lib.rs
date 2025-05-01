#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

mod cache_config;
mod canned_response_config;
mod chain_config;
mod config;
mod cors_config;
mod error_handling_config;
mod load_balancing_config;
mod logging_config;
mod metrics_config;
mod project_config;
mod request_coalescing_config;
mod server_config;
mod upstream_config;
mod upstream_health_checks_config;

pub use cache_config::{CacheConfig, LocalCacheConfig, RedisCacheConfig};
pub use canned_response_config::CannedResponseConfig;
pub use chain_config::ChainConfig;
pub use config::Config;
pub use cors_config::CorsConfig;
pub use error_handling_config::ErrorHandlingConfig;
pub use load_balancing_config::LoadBalancingStrategy;
pub use logging_config::LoggingConfig;
pub use metrics_config::MetricsConfig;
pub use project_config::ProjectConfig;
pub use request_coalescing_config::RequestCoalescingConfig;
pub use server_config::ServerConfig;
pub use upstream_config::UpstreamConfig;
pub use upstream_health_checks_config::UpstreamHealthChecksConfig;
