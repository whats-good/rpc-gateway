use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{body::Incoming, Method, Request, Response, StatusCode};
use regex::Regex;
use std::num::ParseIntError;
use thiserror::Error;
use tracing::info;

use crate::{
    chain::ChainId,
    rpc::{
        RpcInboundRequest, RpcInboundSuccessResponse, RpcOutboundErrorResponse, RpcOutboundRequest,
        RpcOutboundResponse, RpcOutboundSuccessResponse, JSON_RPC_VERSION,
    },
    settings::{Settings, TargetEndpointsForChain},
    target_endpoint::HttpTargetEndpoint,
};

#[derive(Debug, Error)]
pub enum HttpRelayError {
    #[error("Http transport error")]
    HttpTransportError,

    #[error("Inbound parsing error")]
    ParsingError,
}

impl HttpTargetEndpoint {
    pub async fn relay(
        &self,
        req: &RpcOutboundRequest,
    ) -> Result<RpcOutboundResponse, HttpRelayError> {
        let client = reqwest::Client::new(); // TODO: is it safe to pass the client around? can we reuse it? can we keep tcp sockets alive if we share the client around?
        let response = client
            .post(self.url.clone()) // TODO: why do we have to clone the url here?
            .json(&req)
            .send()
            .await
            .or(Err(HttpRelayError::HttpTransportError))?;

        let inbound_success_response = response
            .json::<RpcInboundSuccessResponse>()
            .await
            .or(Err(HttpRelayError::ParsingError))?;

        let outbound_success_response: RpcOutboundSuccessResponse = inbound_success_response.into();
        return Ok(outbound_success_response.into());
    }
}

impl TargetEndpointsForChain {
    pub async fn http_relay(&self, inbound: RpcInboundRequest) -> RpcOutboundResponse {
        info!(
            "Relaying to http node for chain_id: {:?}",
            self.chain.chain_id
        );
        let outbound = RpcOutboundRequest(inbound);
        let mut errors = vec![];
        for endpoint in self.http.iter() {
            match endpoint.relay(&outbound).await {
                Ok(response) => return response,
                Err(e) => {
                    errors.push(e);
                }
            }
        }
        RpcOutboundErrorResponse {
            id: outbound.id,
            error: String::from("no endpoints for chain"), // TODO: turn these into enums
            jsonrpc: JSON_RPC_VERSION,
        }
        .into()
    }
}

#[derive(Error, Debug)]
enum PathToChainError {
    #[error("Invalid regex")]
    RegexError,

    #[error("ParseIntError")]
    ParseError(#[from] ParseIntError),
}

// TODO: write unit test
fn path_to_chain_id(path: &str) -> Result<ChainId, PathToChainError> {
    let re = Regex::new(r"^/(\d+)$").or(Err(PathToChainError::RegexError))?;
    let captures = re.captures(path).ok_or(PathToChainError::RegexError)?;
    let chain_id_match = captures.get(1).ok_or(PathToChainError::RegexError)?;
    let chain_id: ChainId = chain_id_match.as_str().parse::<u64>()?.into();
    Ok(chain_id)
}

impl RpcInboundRequest {
    // TODO: how can we make this a bit more idiomatic (i.e .into().await?)
    async fn try_from_async(value: Request<Incoming>) -> Result<Self, RpcOutboundErrorResponse> {
        let body_bytes = match value.collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(_) => {
                return Err(RpcOutboundErrorResponse {
                    id: None,
                    error: String::from("parse error"), // TODO: turn these into enums
                    jsonrpc: JSON_RPC_VERSION, // TODO: turn this into a constructor and automatically append JSON_RPC_VERSION in there (or even better, skip it from the memory representation and only have in serialization)
                });
            }
        };

        let inbound: RpcInboundRequest = match serde_json::from_slice(&body_bytes.to_vec()) {
            Ok(x) => x,
            Err(_) => {
                return Err(RpcOutboundErrorResponse {
                    id: None,
                    error: String::from("parse error"), // TODO: turn these into enums
                    jsonrpc: JSON_RPC_VERSION,
                });
            }
        };
        return Ok(inbound);
    }
}

async fn handle_chain_id_route(
    chain_id: ChainId,
    settings: &'static Settings,
    req: Request<Incoming>,
) -> RpcOutboundResponse {
    // TODO: could we do a ? operator on the RpcErrorResponse object as if it's a result over RpcOutboundResponse?
    let inbound = match RpcInboundRequest::try_from_async(req).await {
        Ok(inbound) => inbound,
        Err(err) => return err.into(),
    };

    match settings.chains_to_targets.get(&chain_id) {
        Some(target_endpoints_for_chain) => target_endpoints_for_chain.http_relay(inbound).await,
        None => {
            // TODO: consolidate these "no target endpoints found" errors in one place, because it needs to be reused under the lower struct too
            RpcOutboundErrorResponse {
                id: inbound.id,
                error: String::from("settings error: no target endpoints for chain id"), // TODO: turn these into enums
                jsonrpc: JSON_RPC_VERSION,
            }
            .into()
        }
    }
}

pub async fn http_handler(
    req: Request<Incoming>,
    settings: &'static Settings,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    // TODO: dont use boxbody, use regular body instead
    if req.method() != &Method::POST {
        todo!("not a post method")
    }
    let path = req.uri().path();
    let response = match path_to_chain_id(path) {
        Ok(chain_id) => {
            let outbound_response = handle_chain_id_route(chain_id, settings, req).await;
            let outbound_bytes: Bytes = outbound_response
                .try_into()
                .expect("expected to get bytes from outbound"); // TODO: remove this

            // TODO: look into whether these Bytes conversions are necessary, especially right after serialization
            return Ok(Response::new(full(outbound_bytes)));
        }
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    };
    let response = response;

    response
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
