use crate::{load_balancer, request_pool::ChainRequestPool, upstream::Upstream};
use futures::{
    FutureExt,
    future::{self, join_all},
};
use nonempty::NonEmpty;
use rpc_gateway_config::{Config, ProjectConfig};
use rpc_gateway_rpc::{
    error::RpcError,
    request::Request,
    response::{Response, RpcResponse},
};
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, warn};

use crate::chain_handler::ChainHandler;

#[derive(Debug)]
pub struct GatewayRequest {
    pub project_config: ProjectConfig,
    pub key: Option<String>,
    pub chain_id: u64,
    pub req: rpc_gateway_rpc::request::Request,
}

impl GatewayRequest {
    pub fn new(
        project_config: ProjectConfig,
        key: Option<String>,
        chain_id: u64,
        req: Request,
    ) -> Self {
        Self {
            project_config,
            key,
            chain_id,
            req,
        }
    }
}

#[derive(Debug)]
pub struct Gateway {
    handlers: HashMap<u64, ChainHandler>,
    pub config: Config, // TODO: make this private
}

impl Gateway {
    // TODO: this should not be async
    pub async fn new(config: Config) -> Self {
        let mut handlers = HashMap::new();

        // TODO: make sure this chains hashmap is not empty
        for (chain_id, chain_config) in &config.chains {
            let cache = rpc_gateway_cache::cache::from_config(&config.cache, chain_config).await;
            let upstreams = NonEmpty::from_vec(
                chain_config
                    .upstreams
                    .iter()
                    .map(|config| Arc::new(Upstream::new(config.clone(), chain_config.chain)))
                    .collect::<Vec<_>>(),
            )
            .expect("Chain config must have at least one upstream");
            let load_balancer = load_balancer::from_config(
                config.load_balancing.clone(),
                config.upstream_health_checks.clone(),
                upstreams,
            );
            let request_pool = ChainRequestPool::new(config.error_handling.clone(), load_balancer);
            let handler = ChainHandler::new(
                chain_config,
                &config.request_coalescing,
                &config.canned_responses,
                request_pool,
                cache,
            );
            handlers.insert(chain_id.clone(), handler);
        }

        Self { handlers, config }
    }

    // TODO: this should be called by the task manager. it should be async.
    pub async fn start_upstream_health_check_loops(&self) {
        if !self.config.upstream_health_checks.enabled {
            warn!("Upstream health checks are disabled. Not starting health check loops.");
            return;
        }

        debug!("Starting upstream health check loops");

        let health_check_futures: Vec<_> = self
            .handlers
            .values()
            .map(|handler| {
                let manager = handler
                    .request_pool
                    .load_balancer
                    .get_health_check_manager();
                async move {
                    manager.start_upstream_health_check_loop().await;
                }
            })
            .collect();

        join_all(health_check_futures).await;
    }

    pub async fn run_upstream_health_checks_once(&self) {
        let futures = self.handlers.values().map(|handler| {
            let manager = handler
                .request_pool
                .load_balancer
                .get_health_check_manager();

            async move {
                manager.run_health_checks_once().await;
            }
        });

        join_all(futures).await;
    }

    pub async fn handle_request(&self, gateway_request: GatewayRequest) -> Option<Response> {
        let is_authorized = gateway_request.project_config.key == gateway_request.key;

        let chain_handler = match self.handlers.get(&gateway_request.chain_id) {
            Some(chain_handler) => chain_handler,
            None => {
                let error = Response::error(RpcError::internal_error_with("Chain not supported"));
                return Some(error);
            }
        };

        let project_config = &gateway_request.project_config;

        // TODO: track actual incoming requests, and tag them by batch or single
        // separate metrics by inbound vs outbound.

        match (gateway_request.req, is_authorized) {
            (Request::Single(call), true) => chain_handler
                .handle_call(call, project_config)
                .await
                .map(Response::Single),
            (Request::Batch(calls), true) => {
                future::join_all(
                    calls
                        .into_iter()
                        .map(move |call| chain_handler.handle_call(call, project_config)),
                )
                .map(responses_as_batch)
                .await
            }
            (_, false) => {
                warn!("Unauthorized request");
                let error = Response::error(RpcError::internal_error_with("Unauthorized"));
                return Some(error);
            }
        }
    }
}

/// processes batch calls
fn responses_as_batch(outs: Vec<Option<RpcResponse>>) -> Option<Response> {
    let batch: Vec<_> = outs.into_iter().flatten().collect();
    (!batch.is_empty()).then_some(Response::Batch(batch))
}
