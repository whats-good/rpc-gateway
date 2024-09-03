use std::ops::Deref;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

pub const JSON_RPC_VERSION: &'static str = "2.0";

pub type RpcRequestId = Option<u32>; // TODO: is this an appropriate bound on the rpc id?

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

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcInboundErrorPayload {
    pub code: i32,       // TODO: is this big enough?
    pub message: String, // TODO: should we use short string? what about static refs? Maybe we can use `COWs` instead to pass around pre-compiled error messages
}

#[derive(Debug, Deserialize)]
pub struct RpcInboundErrorResponse {
    pub error: RpcInboundErrorPayload,
    pub id: RpcRequestId,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum RpcInboundResponse {
    Success(RpcInboundSuccessResponse),
    Error(RpcInboundErrorResponse),
}

impl From<RpcInboundResponse> for RpcOutboundResponse {
    fn from(value: RpcInboundResponse) -> Self {
        match value {
            RpcInboundResponse::Success(success) => RpcOutboundResponse::Success(success.into()),
            RpcInboundResponse::Error(error) => RpcOutboundResponse::Error(error.into()),
        }
    }
}

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

impl From<RpcOutboundSuccessResponse> for Bytes {
    fn from(value: RpcOutboundSuccessResponse) -> Self {
        let string = serde_json::to_string(&value)
            .expect("Expected the outbound success response to successfully serialize");
        Bytes::from(string)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StaticRpcOutboundErrorPayload {
    pub code: i32,             // TODO: is this big enough?
    pub message: &'static str, // TODO: should we use short string? what about static refs? Maybe we can use `COWs` instead to pass around pre-compiled error messages
}

#[derive(Serialize, Debug)]
pub struct DynamicRpcOutboundErrorPayload {
    pub code: i32,
    pub message: String,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum RpcOutboundErrorPayload {
    Static(StaticRpcOutboundErrorPayload),
    Dynamic(DynamicRpcOutboundErrorPayload),
}

#[derive(Serialize, Debug)]
pub struct RpcOutboundErrorResponse {
    pub error: RpcOutboundErrorPayload,
    pub id: RpcRequestId,
    pub jsonrpc: &'static str, // TODO: see if we can avoid carrying this in the runtime, and only add it during serialization
}

pub const PARSE_ERROR: StaticRpcOutboundErrorPayload = StaticRpcOutboundErrorPayload {
    code: -32_700,
    message: "Parse error",
};

pub const INVALID_REQUEST: StaticRpcOutboundErrorPayload = StaticRpcOutboundErrorPayload {
    code: -32_600,
    message: "Invalid request",
};

pub const INTERNAL_ERROR: StaticRpcOutboundErrorPayload = StaticRpcOutboundErrorPayload {
    code: -32_603,
    message: "Internal error",
};

// TODO: document these custom errors
// TODO: should this just be an INTERNAL_ERROR? Or Resource not found? Or Resource unavailable?
pub const TARGETS_NOT_CONFIGURED: StaticRpcOutboundErrorPayload = StaticRpcOutboundErrorPayload {
    code: -33_000,
    message: "Targets not configured",
};

impl From<RpcInboundErrorResponse> for RpcOutboundErrorResponse {
    fn from(value: RpcInboundErrorResponse) -> Self {
        RpcOutboundErrorResponse {
            id: value.id,
            error: RpcOutboundErrorPayload::Dynamic(DynamicRpcOutboundErrorPayload {
                code: value.error.code,
                message: value.error.message, // TODO: this should be an enum.
            }),
            jsonrpc: JSON_RPC_VERSION,
        }
    }
}

impl From<RpcOutboundErrorResponse> for Bytes {
    fn from(value: RpcOutboundErrorResponse) -> Self {
        let string = serde_json::to_string(&value)
            .expect("Expected outbound error response struct to successfully serialize");
        Bytes::from(string)
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

impl From<RpcOutboundResponse> for Bytes {
    fn from(value: RpcOutboundResponse) -> Self {
        match value {
            RpcOutboundResponse::Success(success_response) => success_response.into(),
            RpcOutboundResponse::Error(error_response) => error_response.into(),
        }
    }
}
