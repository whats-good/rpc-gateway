use crate::load_balancer::LoadBalancer;
use crate::upstream::UpstreamError;
use rpc_gateway_config::ErrorHandlingConfig;
use rpc_gateway_rpc::response::RpcResponse;
use std::sync::Arc;
use tracing::{debug, instrument, warn};

// TODO: maybe request coalescing should be done here?

#[derive(Debug, Clone)]
pub struct ChainRequestPool {
    error_handling: Arc<ErrorHandlingConfig>,
    pub load_balancer: Arc<dyn LoadBalancer>,
}

#[derive(Debug)]
pub enum RequestPoolError {
    NoUpstreamsAvailable,
    UpstreamError(UpstreamError),
}

impl ChainRequestPool {
    pub fn new(error_handling: ErrorHandlingConfig, load_balancer: Arc<dyn LoadBalancer>) -> Self {
        Self {
            error_handling: Arc::new(error_handling),
            load_balancer,
        }
    }

    #[instrument(skip(self))]
    pub async fn forward_request(
        &self,
        raw_call: &serde_json::Value,
    ) -> Result<RpcResponse, RequestPoolError> {
        let upstream = match self.load_balancer.select_upstream() {
            Some(upstream) => upstream,
            None => {
                return Err(RequestPoolError::NoUpstreamsAvailable);
            }
        };
        match &*self.error_handling {
            ErrorHandlingConfig::Retry {
                max_retries,
                retry_delay,
                jitter,
            } => {
                debug!(
                    max_retries = %max_retries,
                    retry_delay = ?retry_delay,
                    jitter = %jitter,
                    "Using retry strategy"
                );
                upstream
                    .forward_with_retry(raw_call, *max_retries, *retry_delay, *jitter)
                    .await
                    .map_err(|err| RequestPoolError::UpstreamError(err))
            }
            ErrorHandlingConfig::FailFast { .. } => {
                debug!("Using fail-fast strategy");
                upstream
                    .forward_once(raw_call)
                    .await
                    .map_err(|err| RequestPoolError::UpstreamError(err))
            }
            ErrorHandlingConfig::CircuitBreaker { .. } => {
                warn!(
                    "Circuit breaker strategy not yet implemented, falling back to single attempt"
                );
                upstream
                    .forward_once(raw_call)
                    .await
                    .map_err(|err| RequestPoolError::UpstreamError(err))
            }
        }
    }
}
