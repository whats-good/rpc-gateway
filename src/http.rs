use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{body::Incoming, Method, Request, Response, StatusCode};
use regex::Regex;
use thiserror::Error;
use tracing::info;

use crate::{
    chain::ChainId,
    rpc::{
        RpcInboundRequest, RpcInboundResponse, RpcOutboundErrorPayload, RpcOutboundErrorResponse,
        RpcOutboundRequest, RpcOutboundResponse, RpcRequestId, StaticRpcOutboundErrorPayload,
        INVALID_REQUEST, JSON_RPC_VERSION, PARSE_ERROR, TARGETS_NOT_CONFIGURED,
    },
    settings::{Settings, TargetEndpointsForChain},
    target_endpoint::HttpTargetEndpoint,
};

#[derive(Debug, Error)]
pub enum HttpRelayError {
    #[error("Http relay transport error")]
    TransportError,

    #[error("Inbound response parsing error")]
    InboundResponseParsingError,
}

impl HttpTargetEndpoint {
    pub async fn relay(
        &self,
        req: &RpcOutboundRequest,
    ) -> Result<RpcInboundResponse, HttpRelayError> {
        let client = reqwest::Client::new(); // TODO: is it safe to pass the client around? can we reuse it? can we keep tcp sockets alive if we share the client around?
        let response = client
            .post(self.url.clone()) // TODO: why do we have to clone the url here?
            .json(&req)
            .send()
            .await
            .or(Err(HttpRelayError::TransportError))?;

        let inbound_response = response
            .json::<RpcInboundResponse>()
            .await
            .or(Err(HttpRelayError::InboundResponseParsingError))?;

        Ok(inbound_response)
    }
}

#[derive(Error, Debug, Clone, Copy)] // TODO: is it okay to implement copy and clone for this enum?
enum RpcOutboundErrorKind {
    #[error("Targets not configured for chain")]
    TargetsNotConfigured(ChainId),
}

pub struct RpcOutboundError {
    request_id: RpcRequestId,
    kind: RpcOutboundErrorKind,
}

impl From<RpcOutboundError> for StaticRpcOutboundErrorPayload {
    fn from(value: RpcOutboundError) -> Self {
        match value.kind {
            RpcOutboundErrorKind::TargetsNotConfigured(_) => TARGETS_NOT_CONFIGURED,
        }
    }
}

impl Into<RpcOutboundResponse> for RpcOutboundError {
    fn into(self) -> RpcOutboundResponse {
        RpcOutboundResponse::Error(RpcOutboundErrorResponse {
            id: self.request_id,
            error: RpcOutboundErrorPayload::Static(self.into()),
            jsonrpc: JSON_RPC_VERSION,
        })
    }
}

impl TargetEndpointsForChain {
    pub async fn http_relay(
        &self,
        inbound: RpcInboundRequest,
    ) -> Result<RpcOutboundResponse, RpcOutboundError> {
        info!(
            "Relaying to http node for chain_id: {:?}",
            self.chain.chain_id
        );
        let outbound = RpcOutboundRequest(inbound);
        let mut errors = vec![];
        for endpoint in self.http.iter() {
            match endpoint.relay(&outbound).await {
                // TODO: configure a setting that allows relaying the request to another endpoint if its a valid error response
                Ok(response) => return Ok(response.into()),
                Err(e) => {
                    errors.push(e);
                }
            }
        }
        Err(RpcOutboundError {
            kind: RpcOutboundErrorKind::TargetsNotConfigured(self.chain.chain_id),
            request_id: outbound.id,
        })
    }
}

#[derive(Error, Debug)]
enum InboundRpcError {
    #[error("Hyper bytes collect error")]
    HyperBytes(#[from] hyper::Error),

    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),

    #[error("Route format error")]
    RouteFormat,
}

impl Into<StaticRpcOutboundErrorPayload> for InboundRpcError {
    fn into(self) -> StaticRpcOutboundErrorPayload {
        match self {
            InboundRpcError::RouteFormat | InboundRpcError::HyperBytes(_) => INVALID_REQUEST,
            InboundRpcError::Serialization(_) => PARSE_ERROR,
        }
    }
}

impl Into<RpcOutboundErrorResponse> for InboundRpcError {
    fn into(self) -> RpcOutboundErrorResponse {
        RpcOutboundErrorResponse {
            error: RpcOutboundErrorPayload::Static(self.into()),
            id: None,
            jsonrpc: JSON_RPC_VERSION,
        }
    }
}

impl Into<RpcOutboundResponse> for InboundRpcError {
    fn into(self) -> RpcOutboundResponse {
        let err_response: RpcOutboundErrorResponse = self.into();
        err_response.into()
    }
}

impl RpcInboundRequest {
    async fn try_from_async(value: Request<Incoming>) -> Result<Self, InboundRpcError> {
        let body_bytes = value.collect().await?;
        let inbound: RpcInboundRequest = serde_json::from_slice(&body_bytes.to_bytes().to_vec())?;
        return Ok(inbound);
    }
}

pub struct HttpHandler {
    pub req: Request<Incoming>,
    pub settings: &'static Settings,
}

impl HttpHandler {
    async fn handle_chain_id_route(self, chain_id: ChainId) -> RpcOutboundResponse {
        let inbound = match RpcInboundRequest::try_from_async(self.req).await {
            Ok(inbound) => inbound,
            Err(err) => return err.into(),
        };

        // TODO: what's a good way to add some tracing & logs while constructing the RpcOutboundResponse from error enums?
        match self.settings.chains_to_targets.get(&chain_id) {
            Some(target_endpoints_for_chain) => target_endpoints_for_chain
                .http_relay(inbound)
                .await
                .unwrap_or_else(|e| e.into()),
            None => RpcOutboundError {
                kind: RpcOutboundErrorKind::TargetsNotConfigured(chain_id),
                request_id: inbound.id,
            }
            .into(),
        }
    }

    pub async fn handle(self) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        // TODO: dont use boxbody, use regular body instead
        if self.req.method() != &Method::POST {
            todo!("not a post method")
        }
        let path = self.req.uri().path();
        let response = match Self::path_to_chain_id(path) {
            Ok(chain_id) => {
                let outbound_response = self.handle_chain_id_route(chain_id).await;
                let outbound_bytes: Bytes = outbound_response.into();

                return Ok(Response::new(Self::full(outbound_bytes)));
            }
            _ => {
                let mut not_found = Response::new(Self::empty());
                *not_found.status_mut() = StatusCode::NOT_FOUND;
                Ok(not_found)
            }
        };
        let response = response;

        response
    }

    fn path_to_chain_id(path: &str) -> Result<ChainId, InboundRpcError> {
        let re = Regex::new(r"^/(\d+)$").unwrap();
        let captures = re.captures(path).ok_or(InboundRpcError::RouteFormat)?;
        let chain_id_match = captures.get(1).ok_or(InboundRpcError::RouteFormat)?;
        let chain_id: ChainId = chain_id_match
            .as_str()
            .parse::<u64>()
            .map_err(|_| InboundRpcError::RouteFormat)?
            .into();
        info!("request received for {chain_id:?}");
        Ok(chain_id)
    }

    fn empty() -> BoxBody<Bytes, hyper::Error> {
        Empty::<Bytes>::new()
            .map_err(|never| match never {})
            .boxed()
    }

    fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
        Full::new(chunk.into())
            .map_err(|never| match never {})
            .boxed()
    }
}
