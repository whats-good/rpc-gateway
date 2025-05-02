use crate::lazy_request::PreservedSingleCall;
use crate::request_pool::{ChainRequestPool, RequestPoolError};
use crate::upstream::UpstreamError;
use alloy_primitives::hex;
use dashmap::DashMap;
use futures::FutureExt;
use futures::future::Shared;
use metrics::{Label, counter, histogram};
use rpc_gateway_cache::cache::RpcCache;
use rpc_gateway_config::{
    CannedResponseConfig, ChainConfig, ProjectConfig, RequestCoalescingConfig,
};
use rpc_gateway_eth::eth::EthRequest;
use rpc_gateway_rpc::error::RpcError;
use rpc_gateway_rpc::request::{RpcCall, RpcMethodCall};
use rpc_gateway_rpc::response::{ResponseResult, RpcResponse};
use std::borrow::Cow;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{debug, error, instrument, warn};

#[derive(Debug, Clone)]
enum ChainHandlerResponseSource {
    Upstream,
    Coalesced,
    Cached,
    Canned,
    PreUpstreamError,
}

impl From<ChainHandlerResponseSource> for Cow<'static, str> {
    fn from(source: ChainHandlerResponseSource) -> Self {
        Cow::Borrowed(match source {
            ChainHandlerResponseSource::Upstream => "upstream",
            ChainHandlerResponseSource::Coalesced => "coalesced",
            ChainHandlerResponseSource::Cached => "cached",
            ChainHandlerResponseSource::Canned => "canned",
            ChainHandlerResponseSource::PreUpstreamError => "pre_upstream_error",
        })
    }
}

#[derive(Debug, Clone)]
struct ChainHandlerResponse {
    response_source: ChainHandlerResponseSource,
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
    pub cache: Option<Arc<RpcCache>>, // TODO: is this the right way to do this?
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
        match call.parsed {
            RpcCall::MethodCall(call) => Some(self.on_method_call(call, project_config).await),
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

    // TODO: how does anvil convert from RpcMethodCall to EthRequest? Do they also parse-down to json first?
    #[instrument(name = "on_method_call", fields(method = %call.method, params = ?call.params), skip(self, call, project_config))]
    async fn on_method_call(
        &self,
        call: RpcMethodCall,
        project_config: &ProjectConfig,
    ) -> RpcResponse {
        let chain_id = self.chain_config.chain.id().to_string();

        let start_time = std::time::Instant::now();

        let chain_handler_response = self.on_request(&call, project_config).await;

        debug!(
          chain_id = chain_id,
          rpc_method = ?call.method,
          response_success = ?chain_handler_response.response_result,
          response_source = ?chain_handler_response.response_source,
          gateway_project = ?project_config.name,
          "RPC response ready",
        );

        let source: Cow<'static, str> = chain_handler_response.response_source.into(); // TODO: is this the right way to do this?
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

        counter!("rpc_responses_total",
          "chain_id" => chain_id.clone(),
          "rpc_method" => call.method.clone(),
          "response_success" => success,
          "response_source" => source.clone(),
          "gateway_project" => project_config.name.clone(), // TODO: this should come from the span
        )
        .increment(1);

        let response_result = chain_handler_response.response_result;

        let duration = start_time.elapsed();
        histogram!("method_call_latency_seconds",
          "chain_id" => chain_id.clone(),
          "rpc_method" => call.method.clone(),
          "response_success" => success,
          "response_source" => source.clone(),
          "gateway_project" => project_config.name.clone(),
        )
        .record(duration.as_secs_f64());

        RpcResponse::new(call.id, response_result)
    }

    async fn try_canned_response(&self, req: &EthRequest) -> Option<ResponseResult> {
        if !self.canned_responses_config.enabled {
            return None;
        }

        match req {
            EthRequest::Web3ClientVersion(_)
                if self.canned_responses_config.methods.web3_client_version =>
            {
                Some(CANNED_RESPONSE_CLIENT_VERSION.clone())
            }
            EthRequest::EthChainId(_) if self.canned_responses_config.methods.eth_chain_id => {
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
        raw_call: serde_json::Value,
        req: Result<EthRequest, serde_json::Error>,
    ) -> ChainHandlerResponse {
        // TODO: is it safe to unwrap here?
        let coalescing_key = match &req {
            Ok(req) => serde_json::to_string(&req).unwrap(),
            Err(_) => serde_json::to_string(&raw_call).unwrap(),
        };

        let (outer_fut, coalesced) = {
            let request_pool = self.request_pool.clone();
            let raw_call = raw_call.clone();
            let cache = self.cache.clone();
            let in_flight_requests = self.in_flight_requests.clone();

            match self.in_flight_requests.entry(coalescing_key) {
                dashmap::Entry::Occupied(e) => (e.get().clone(), true),
                dashmap::Entry::Vacant(e) => {
                    // TODO: consider reusing the cache key here.
                    let inner_fut = cache_then_upstream(request_pool, cache, raw_call, req)
                        .boxed()
                        .shared();

                    let coalescing_key_for_removal = e.key().clone();

                    // TODO: consider capping the dashmap size

                    tokio::spawn(async move {
                        in_flight_requests.remove(&coalescing_key_for_removal);
                    });

                    e.insert(inner_fut.clone());
                    (inner_fut, false)
                }
            }
        };

        // Await the future
        let result = outer_fut.await;

        if coalesced {
            return ChainHandlerResponse {
                response_source: ChainHandlerResponseSource::Coalesced,
                response_result: result.response_result,
            };
        }

        result
    }
    #[inline]
    fn track_eth_call_requests(
        &self,
        req: &Result<EthRequest, serde_json::Error>,
        project_config: &ProjectConfig,
    ) {
        // TODO: make this configurable. no need to track this metric if the user doesn't want to.
        // TODO: track whether this response was successful.
        // TODO: track response_source just like for the other rpc responses.
        match &req {
            Ok(req) => {
                if let EthRequest::EthCall(call, _, _) = req {
                    let to = call
                        .inner
                        .to
                        .and_then(|to| to.to().copied())
                        .map(|to| to.to_string());
                    let input = call.inner.input.clone();
                    let selector = input.data.and_then(|x| x.get(0..4).map(hex::encode));
                    let from = call.inner.from.map(|x| x.to_string());

                    let mut labels = vec![
                        Label::new("chain_id", self.chain_config.chain.id().to_string()),
                        Label::new("gateway_project", project_config.name.clone()),
                    ];

                    debug!(?selector, ?from, ?to, "eth call request");

                    if let Some(to) = to {
                        labels.push(Label::new("to", to));
                    }

                    if let Some(selector) = selector {
                        labels.push(Label::new("selector", selector));
                    }

                    counter!("eth_call_requests_total", labels).increment(1);
                }
            }
            Err(_) => {}
        }
    }

    async fn on_request(
        &self,
        call: &RpcMethodCall,
        project_config: &ProjectConfig,
    ) -> ChainHandlerResponse {
        let raw_call = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": call.method.clone(),
            "params": call.params
        });
        // TODO: shouldn't there be an easier way to convert RpcMethodCall to EthRequest?
        let req = serde_json::from_value::<EthRequest>(raw_call.clone());

        self.track_eth_call_requests(&req, project_config);

        let canned_response = match &req {
            Ok(req) => self.try_canned_response(req).await,
            Err(_) => None,
        };

        if let Some(response_result) = canned_response {
            // TODO: may want to cache canned responses if they are expensive to generate
            return ChainHandlerResponse {
                response_source: ChainHandlerResponseSource::Canned,
                response_result,
            };
        }

        if self.request_coalescing_config.should_coalesce(&call.method) {
            self.handle_request_with_coalescing(raw_call, req).await
        } else {
            cache_then_upstream(self.request_pool.clone(), self.cache.clone(), raw_call, req).await
        }
    }
}

async fn try_cache_write(cache: &Option<Arc<RpcCache>>, req: &EthRequest, res: &ResponseResult) {
    let cache = match cache {
        Some(cache) => cache,
        None => return,
    };
    let cache_ttl = match cache.get_ttl(&req) {
        Some(cache_ttl) => cache_ttl,
        None => return,
    };
    let successful_response_result = match res {
        ResponseResult::Success(res) => res,
        ResponseResult::Error(_) => {
            debug!(?req, "method returned error, not caching");
            return;
        }
    };
    debug!(?req, "caching response");
    cache
        .insert(&req, successful_response_result, cache_ttl)
        .await;
}

async fn try_cache_read(cache: &Option<Arc<RpcCache>>, req: &EthRequest) -> Option<ResponseResult> {
    let cache = match cache {
        Some(cache) => cache,
        None => return None,
    };

    let cache_ttl = cache.get_ttl(&req);

    if cache_ttl.is_some() {
        debug!(?req, "method is cacheable");
        if let Some(response) = cache.get(&req).await {
            debug!(?req, "cache hit");
            return Some(ResponseResult::Success(response));
        } else {
            debug!(?req, "cache miss");
        }
    } else {
        debug!(?req, "method is not cacheable");
    }
    None
}

async fn forward_to_upstream(
    request_pool: Arc<ChainRequestPool>,
    raw_call: serde_json::Value,
) -> ChainHandlerResponse {
    // TODO: come up with proxy specific error codes.
    // TODO: metrics and logs should distinguish between legal rpc error responses returned from upstreams,
    // and errors generated by the proxy itself.
    let error = match request_pool.forward_request(&raw_call).await {
        Ok(response) => {
            return ChainHandlerResponse {
                response_source: ChainHandlerResponseSource::Upstream,
                response_result: response.result,
            };
        }
        Err(e) => e,
    };

    let response = match error {
        RequestPoolError::NoUpstreamsAvailable => ChainHandlerResponse {
            response_source: ChainHandlerResponseSource::PreUpstreamError,
            response_result: ResponseResult::Error(RpcError::internal_error_with(
                "No upstreams available",
            )),
        },
        RequestPoolError::UpstreamError(UpstreamError::RequestError(_)) => ChainHandlerResponse {
            response_source: ChainHandlerResponseSource::PreUpstreamError,
            response_result: ResponseResult::Error(RpcError::internal_error_with(
                "Could not forward request to upstream",
            )),
        },
        RequestPoolError::UpstreamError(UpstreamError::ResponseError(_)) => ChainHandlerResponse {
            response_source: ChainHandlerResponseSource::Upstream,
            response_result: ResponseResult::Error(RpcError::internal_error_with(
                "Upstream response error",
            )),
        },
    };

    // TODO: add better logging and fields for the error. also add metrics and counters.
    error!(?error, "request pool error");

    response
}

async fn cache_then_upstream(
    request_pool: Arc<ChainRequestPool>,
    cache: Option<Arc<RpcCache>>,
    raw_call: serde_json::Value,
    req: Result<EthRequest, serde_json::Error>,
) -> ChainHandlerResponse {
    let req = match req {
        Ok(req) => req,
        Err(err) => {
            warn!(
                ?err,
                "Failed to parse eth request. Forwarding to upstream without caching."
            );
            return forward_to_upstream(request_pool, raw_call).await;
        }
    };

    if let Some(response_result) = try_cache_read(&cache, &req).await {
        return ChainHandlerResponse {
            response_source: ChainHandlerResponseSource::Cached,
            response_result,
        };
    }

    let response = forward_to_upstream(request_pool, raw_call).await;

    if matches!(
        response.response_source,
        ChainHandlerResponseSource::Upstream
    ) {
        try_cache_write(&cache, &req, &response.response_result).await;
    }

    response
}
