use std::ops::Deref;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

pub const JSON_RPC_VERSION: &'static str = "2.0";

type RpcRequestId = Option<u32>; // TODO: is this an appropriate bound on the rpc id?

#[derive(Deserialize, Debug, Serialize)]
pub struct RpcInboundRequest {
    pub id: RpcRequestId, // TODO: write tests for empty id and null ids
    pub method: String,
    pub params: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct RpcOutboundRequest(pub RpcInboundRequest);

impl Deref for RpcOutboundRequest {
    type Target = RpcInboundRequest;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize)]
pub struct RpcInboundSuccessResponse {
    pub result: String,
    pub id: RpcRequestId,
}

#[derive(Debug, Deserialize)]
pub struct RpcInboundErrorResponse {
    pub error: String, // TODO: parse the error
    pub id: RpcRequestId,
}

// TODO: use enum flattening
// #[derive(Deserialize, Debug)]
// enum RpcInboundResponse {
//     success(RpcInboundResponseSuccess),

//     error(RpcInboundResponseError),
// }

#[derive(Serialize, Debug)]
pub struct RpcOutboundSuccessResponse {
    pub result: String,
    pub id: RpcRequestId, // TODO: outbound ids should serialize to "null" if they don't exist
    pub jsonrpc: &'static str,
}

impl From<RpcInboundSuccessResponse> for RpcOutboundSuccessResponse {
    fn from(value: RpcInboundSuccessResponse) -> Self {
        RpcOutboundSuccessResponse {
            id: value.id,
            result: value.result,
            jsonrpc: JSON_RPC_VERSION,
        }
    }
}

impl TryFrom<RpcOutboundSuccessResponse> for Bytes {
    type Error = serde_json::Error;

    fn try_from(value: RpcOutboundSuccessResponse) -> Result<Self, Self::Error> {
        let string = serde_json::to_string(&value)?;
        let bytes = Bytes::from(string);
        Ok(bytes)
    }
}

#[derive(Serialize, Debug)]
pub struct RpcOutboundErrorResponse {
    pub error: String,
    pub id: RpcRequestId,
    pub jsonrpc: &'static str, // TODO: see if we can avoid carrying this in the runtime, and only add it during serialization
}

impl From<RpcInboundErrorResponse> for RpcOutboundErrorResponse {
    fn from(value: RpcInboundErrorResponse) -> Self {
        RpcOutboundErrorResponse {
            id: value.id,
            error: value.error,
            jsonrpc: JSON_RPC_VERSION,
        }
    }
}

impl TryFrom<RpcOutboundErrorResponse> for Bytes {
    type Error = serde_json::Error;

    fn try_from(value: RpcOutboundErrorResponse) -> Result<Self, Self::Error> {
        let string = serde_json::to_string(&value)?;
        let bytes = Bytes::from(string);
        Ok(bytes)
    }
}

#[derive(Debug)]
pub enum RpcOutboundResponse {
    Success(RpcOutboundSuccessResponse),
    Error(RpcOutboundErrorResponse),
}

impl From<RpcOutboundSuccessResponse> for RpcOutboundResponse {
    fn from(value: RpcOutboundSuccessResponse) -> Self {
        RpcOutboundResponse::Success(value)
    }
}

impl From<RpcOutboundErrorResponse> for RpcOutboundResponse {
    fn from(value: RpcOutboundErrorResponse) -> Self {
        RpcOutboundResponse::Error(value)
    }
}

impl TryFrom<RpcOutboundResponse> for Bytes {
    type Error = serde_json::Error;

    fn try_from(value: RpcOutboundResponse) -> Result<Self, Self::Error> {
        match value {
            RpcOutboundResponse::Success(success_response) => success_response.try_into(),
            RpcOutboundResponse::Error(error_response) => error_response.try_into(),
        }
    }
}
