use serde::{Deserialize, Serialize};
use simd_json::OwnedValue;
use std::fmt;

/// A JSON-RPC request object, a method call
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RpcMethodCall {
    /// The version of the protocol
    pub jsonrpc: Version,
    /// The name of the method to execute
    pub method: String,
    /// An array or object containing the parameters to be passed to the function.
    #[serde(default = "no_params")]
    pub params: RequestParams,
    /// The identifier for this request issued by the client,
    /// An [Id] must be a String, null or a number.
    /// If missing it's considered a notification in [Version::V2]
    pub id: Id,
}

impl RpcMethodCall {
    pub fn id(&self) -> Id {
        self.id.clone()
    }
}

/// Represents a JSON-RPC request which is considered a notification (missing [Id] optional
/// [Version])
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RpcNotification {
    pub jsonrpc: Option<Version>,
    pub method: String,
    #[serde(default = "no_params")]
    pub params: RequestParams,
}

/// Representation of a single JSON-RPC call
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcCall {
    /// the RPC method to invoke
    MethodCall(RpcMethodCall),
    /// A notification (no [Id] provided)
    Notification(RpcNotification),
    /// Invalid call
    Invalid {
        /// id or [Id::Null]
        #[serde(default = "null_id")]
        id: Id,
    },
}

/// Represents a JSON-RPC request.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Request {
    /// single json rpc request [RpcCall]
    Single(RpcCall),
    /// batch of several requests
    Batch(Vec<RpcCall>),
}

/// Request parameters
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum RequestParams {
    /// no parameters provided
    None,
    /// An array of JSON values
    Array(Vec<OwnedValue>),
    /// a map of JSON values
    Object(OwnedValue),
}

fn no_params() -> RequestParams {
    RequestParams::None
}

/// Represents the version of the RPC protocol
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Version {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Id {
    String(String),
    Number(i64),
    Null,
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => s.fmt(f),
            Self::Number(n) => n.fmt(f),
            Self::Null => f.write_str("null"),
        }
    }
}

fn null_id() -> Id {
    Id::Null
}
