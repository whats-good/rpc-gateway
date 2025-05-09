use crate::lazy_request::{PreservedMethodCall, PreservedSingleCall};
use crate::request_pool::{ChainRequestPool, RequestPoolError};
use bytes::Bytes;
use dashmap::DashMap;
use futures::FutureExt;
use futures::future::Shared;
use metrics::{counter, histogram};
use rpc_gateway_cache::cache::RpcCache;
use rpc_gateway_config::{
    CannedResponseConfig, ChainConfig, ProjectConfig, RequestCoalescingConfig,
};
use rpc_gateway_eth::eth::EthRequest;
use rpc_gateway_rpc::error::RpcError;
use rpc_gateway_rpc::request::RpcCall;
use rpc_gateway_rpc::response::{ResponseResult, RpcResponse};
use rpc_gateway_upstream::upstream::UpstreamError;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, instrument, warn};

const RESPONSE_SOURCE_UPSTREAM: &str = "upstream";
const RESPONSE_SOURCE_COALESCED: &str = "coalesced";
const RESPONSE_SOURCE_CACHED: &str = "cached";
const RESPONSE_SOURCE_CANNED: &str = "canned";
const RESPONSE_SOURCE_PRE_UPSTREAM_ERROR: &str = "pre_upstream_error";
const RESPONSE_SOURCE_UNSUPPORTED: &str = "unsupported";

struct CacheIntent {
    key: String,
    ttl: Duration,
    cache: Arc<RpcCache>,
}

impl CacheIntent {
    async fn insert(self, res: &serde_json::Value) {
        self.cache.insert(self.key, res, self.ttl).await;
    }

    async fn get(&self) -> Option<serde_json::Value> {
        self.cache.get(&self.key).await
    }
}

#[derive(Debug, Clone)]
struct ChainHandlerResponse {
    response_source: &'static str,
    response_result: ResponseResult,
}

type BoxedResponseFuture = Pin<Box<dyn Future<Output = ChainHandlerResponse> + Send>>;
type SharedResponseFuture = Shared<BoxedResponseFuture>;

#[derive(Debug)]
pub struct ChainHandler {
    pub chain_config: Arc<ChainConfig>,
    pub request_coalescing_config: RequestCoalescingConfig,
    pub canned_responses_config: CannedResponseConfig,
    pub request_pool: Arc<ChainRequestPool>,
    pub cache: Option<Arc<RpcCache>>,
    in_flight_requests: Arc<DashMap<String, SharedResponseFuture>>, // TODO: is there a max size here? what's the limit?
}
use std::sync::LazyLock;

static CANNED_RESPONSE_CLIENT_VERSION: LazyLock<ResponseResult> = LazyLock::new(|| {
    let version = env!("CARGO_PKG_VERSION");
    ResponseResult::Success(serde_json::json!(format!("RPC-Gateway/{}", version)))
});

impl ChainHandler {
    pub fn new(
        chain_config: &ChainConfig,
        request_coalescing_config: &RequestCoalescingConfig,
        canned_responses_config: &CannedResponseConfig,
        request_pool: ChainRequestPool,
        cache: Option<RpcCache>,
    ) -> Self {
        Self {
            chain_config: Arc::new(chain_config.clone()),
            request_pool: Arc::new(request_pool),
            cache: cache.map(Arc::new),
            request_coalescing_config: request_coalescing_config.clone(),
            canned_responses_config: canned_responses_config.clone(),
            in_flight_requests: Arc::new(DashMap::new()),
        }
    }

    /// handle a single RPC method call
    pub async fn handle_call(
        &self,
        call: PreservedSingleCall,
        project_config: &ProjectConfig,
    ) -> Option<RpcResponse> {
        match call.deserialized {
            RpcCall::MethodCall(method_call) => Some(
                self.on_method_call(
                    PreservedMethodCall {
                        deserialized: method_call,
                        raw: call.raw,
                    },
                    project_config,
                )
                .await,
            ),
            RpcCall::Notification(notification) => {
                // TODO: handle notifications
                warn!(target: "rpc", method = ?notification.method, "received rpc notification");
                None
            }
            RpcCall::Invalid { id } => {
                warn!(target: "rpc", ?id,  "invalid rpc call");
                Some(RpcResponse::invalid_request(id))
            }
        }
    }

    #[instrument(name = "on_method_call", fields(method = %call.deserialized.method, params = ?call.deserialized.params), skip(self, call, project_config))]
    async fn on_method_call(
        &self,
        call: PreservedMethodCall,
        project_config: &ProjectConfig,
    ) -> RpcResponse {
        let chain_id = self.chain_config.chain.id().to_string();

        let start_time = std::time::Instant::now();

        // TODO: get the project config from the span
        let chain_handler_response = self.on_request(&call).await;

        debug!(
          chain_id = chain_id,
          rpc_method = ?call.deserialized.method,
          response_success = ?chain_handler_response.response_result,
          response_source = ?chain_handler_response.response_source,
          gateway_project = ?project_config.name,
          "RPC response ready",
        );

        let source = chain_handler_response.response_source;
        let success = match &chain_handler_response.response_result {
            ResponseResult::Success(_) => "true",
            ResponseResult::Error(err) => {
                // TODO: start a new counter for upstream errors, and label by status code and url
                warn!(
                    code = ?err.code,
                    message = ?err.message,
                    data = ?err.data,
                    "method returned error"
                );
                "false"
            }
        };

        counter!("method_call_response_total",
          "chain_id" => chain_id.clone(),
          "rpc_method" => call.deserialized.method.clone(),
          "response_success" => success,
          "response_source" => source,
          "gateway_project" => project_config.name.clone(), // TODO: this should come from the span
        )
        .increment(1);

        let response_result = chain_handler_response.response_result;

        let duration = start_time.elapsed();
        histogram!("method_call_response_latency_seconds",
          "chain_id" => chain_id.clone(),
          "rpc_method" => call.deserialized.method.clone(),
          "response_success" => success,
          "response_source" => source,
          "gateway_project" => project_config.name.clone(),
        )
        .record(duration.as_secs_f64());

        RpcResponse::new(call.deserialized.id, response_result)
    }

    async fn try_canned_response(
        &self,
        req: &Result<EthRequest, serde_json::Error>,
    ) -> Option<ResponseResult> {
        let req = match req {
            Ok(req) => req,
            Err(_) => return None,
        };

        if !self.canned_responses_config.enabled {
            return None;
        }

        match req {
            EthRequest::Web3ClientVersion { .. }
                if self.canned_responses_config.methods.web3_client_version =>
            {
                Some(CANNED_RESPONSE_CLIENT_VERSION.clone())
            }
            EthRequest::EthChainId { .. } if self.canned_responses_config.methods.eth_chain_id => {
                Some(ResponseResult::Success(serde_json::json!(format!(
                    "0x{:x}",
                    self.chain_config.chain.id()
                ))))
            }
            // EthRequest::Web3Sha3(bytes) => todo!(), TODO: self-implement
            // EthRequest::EthNetworkId(_) => todo!(), TODO: self-implement
            _ => None,
        }
    }

    async fn handle_request_with_coalescing(
        &self,
        call: &PreservedMethodCall,
        cache_intent: Option<CacheIntent>,
    ) -> ChainHandlerResponse {
        let coalescing_key = match &cache_intent {
            Some(cache_intent) => cache_intent.key.clone(),
            None => {
                let method = call.deserialized.method.clone(); // TODO: should this be Cow instead?
                let params = serde_json::to_string(&call.deserialized.params).unwrap();
                format!("{}:{}", method, params)
            }
        };

        // TODO: consider capping the dashmap size
        let (outer_fut, coalesced) = {
            match self.in_flight_requests.entry(coalescing_key.clone()) {
                dashmap::Entry::Occupied(e) => (e.get().clone(), true),
                dashmap::Entry::Vacant(e) => {
                    let request_pool = self.request_pool.clone();
                    let raw_call = call.raw.clone();
                    let inner_fut: Shared<
                        Pin<Box<dyn Future<Output = ChainHandlerResponse> + Send>>,
                    > = cache_then_upstream(request_pool, raw_call, cache_intent)
                        .boxed()
                        .shared();

                    counter!("debug_in_flight_request", "action" => "added").increment(1);
                    e.insert(inner_fut.clone());

                    (inner_fut, false)
                }
            }
        };

        if !coalesced {
            // TODO: check if there's a race condition that could prevent this spawn from being executed. otherwise we'll have a memory leak.
            let outer_fut_clone = outer_fut.clone();
            let in_flight_requests_clone = self.in_flight_requests.clone();
            tokio::spawn(async move {
                outer_fut_clone.await;
                in_flight_requests_clone.remove(&coalescing_key);
                counter!("debug_in_flight_request", "action" => "removed").increment(1);
            });
        }

        let result = outer_fut.await;

        if coalesced {
            return ChainHandlerResponse {
                response_source: RESPONSE_SOURCE_COALESCED,
                response_result: result.response_result,
            };
        }

        result
    }

    #[inline]
    fn get_cache_intent(&self, req: &Result<EthRequest, serde_json::Error>) -> Option<CacheIntent> {
        let cache = match &self.cache {
            Some(cache) => cache,
            None => return None,
        };

        let req = match req {
            Ok(req) => req,
            Err(err) => {
                error!(?err, ?req, "failed to parse eth request. not caching.");
                return None;
            }
        };

        let ttl = cache.get_ttl(&req)?;
        let key = req.get_key();

        // TODO: missed oppotrunity: if the request is coalescable, but not cacheable, we'd be forcing the
        // coalescing key compute to use the raw call instead of eth request.

        Some(CacheIntent {
            key,
            ttl,
            cache: cache.clone(),
        })
    }

    #[cold]
    fn try_unsupported_response(&self, call: &PreservedMethodCall) -> Option<ChainHandlerResponse> {
        if call.deserialized.method == "eth_newBlockFilter"
            || call.deserialized.method == "eth_newPendingTransactionFilter"
        {
            Some(ChainHandlerResponse {
                response_source: RESPONSE_SOURCE_UNSUPPORTED,
                response_result: ResponseResult::Error(RpcError::method_not_found()), // TODO: this should technically be an unsupported method error
            })
        } else {
            None
        }
    }

    async fn on_request(&self, call: &PreservedMethodCall) -> ChainHandlerResponse {
        // TODO: shouldn't there be an easier way to convert RpcMethodCall to EthRequest?

        if let Some(response) = self.try_unsupported_response(call) {
            return response;
        }

        let req = serde_json::from_slice::<EthRequest>(&call.raw);

        // TODO: add this back
        // self.track_eth_call_requests(&req, project_config);

        if let Some(response_result) = self.try_canned_response(&req).await {
            // TODO: may want to cache canned responses if they are expensive to generate
            return ChainHandlerResponse {
                response_source: RESPONSE_SOURCE_CANNED,
                response_result,
            };
        }

        let cache_intent = self.get_cache_intent(&req);

        if self
            .request_coalescing_config
            .should_coalesce(&call.deserialized.method)
        {
            self.handle_request_with_coalescing(&call, cache_intent)
                .await
        } else {
            cache_then_upstream(self.request_pool.clone(), call.raw.clone(), cache_intent).await
        }
    }
}

async fn forward_to_upstream(
    request_pool: Arc<ChainRequestPool>,
    raw_call: Bytes,
) -> ChainHandlerResponse {
    // TODO: come up with proxy specific error codes.
    // TODO: metrics and logs should distinguish between legal rpc error responses returned from upstreams,
    // and errors generated by the proxy itself.
    let error = match request_pool.forward_request(raw_call).await {
        Ok(response) => {
            return ChainHandlerResponse {
                response_source: RESPONSE_SOURCE_UPSTREAM,
                response_result: response.result,
            };
        }
        Err(e) => e,
    };

    let response = match error {
        RequestPoolError::NoUpstreamsAvailable => {
            error!("no upstreams available");
            ChainHandlerResponse {
                response_source: RESPONSE_SOURCE_PRE_UPSTREAM_ERROR,
                response_result: ResponseResult::Error(RpcError::internal_error_with(
                    "No upstreams available",
                )),
            }
        }
        RequestPoolError::UpstreamError(UpstreamError::RequestError(_)) => {
            error!("upstream request error");
            ChainHandlerResponse {
                response_source: RESPONSE_SOURCE_PRE_UPSTREAM_ERROR,
                response_result: ResponseResult::Error(RpcError::internal_error_with(
                    "Could not forward request to upstream",
                )),
            }
        }
        RequestPoolError::UpstreamError(UpstreamError::ResponseError(_)) => {
            error!("upstream response error");
            ChainHandlerResponse {
                response_source: RESPONSE_SOURCE_UPSTREAM,
                response_result: ResponseResult::Error(RpcError::internal_error_with(
                    "Upstream response error",
                )),
            }
        }
    };

    // TODO: add better logging and fields for the error. also add metrics and counters.
    warn!(?error, "request pool error");

    response
}

async fn cache_then_upstream(
    request_pool: Arc<ChainRequestPool>,
    raw_call: Bytes,
    cache_intent: Option<CacheIntent>,
) -> ChainHandlerResponse {
    if let Some(cache_intent) = &cache_intent {
        if let Some(response_result) = cache_intent.get().await {
            return ChainHandlerResponse {
                response_source: RESPONSE_SOURCE_CACHED,
                response_result: ResponseResult::Success(response_result),
            };
        }
    }

    let start_time = std::time::Instant::now();
    let response = forward_to_upstream(request_pool, raw_call).await;
    let duration = start_time.elapsed();

    // TODO: add labels
    histogram!("upstream_response_latency_seconds").record(duration.as_secs_f64());

    if matches!(response.response_source, RESPONSE_SOURCE_UPSTREAM) {
        if let Some(cache_intent) = cache_intent {
            if let ResponseResult::Success(response_result) = &response.response_result {
                cache_intent.insert(response_result).await;
            }
        }
    }

    response
}
