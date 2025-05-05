use bytes::Bytes;
use rpc_gateway_rpc::request::{RpcCall, RpcMethodCall};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PreservedSingleCall {
    pub raw: Bytes,
    pub deserialized: RpcCall,
}

#[derive(Debug)]
pub struct PreservedMethodCall {
    pub raw: Bytes,
    pub deserialized: RpcMethodCall,
}

#[derive(Debug, Serialize)]
pub enum PreservedRequest {
    Single(PreservedSingleCall),
    Batch(Vec<PreservedSingleCall>),
}

impl TryFrom<Bytes> for PreservedSingleCall {
    type Error = ();

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let mut vec = value.to_vec();
        let deserialized = simd_json::from_slice(&mut vec).map_err(|_| ())?;
        Ok(PreservedSingleCall {
            raw: value,
            deserialized,
        })
    }
}

impl TryFrom<RpcCall> for PreservedSingleCall {
    type Error = ();

    fn try_from(value: RpcCall) -> Result<Self, Self::Error> {
        let json_bytes = simd_json::to_vec(&value).map_err(|_| ())?;
        let bytes = Bytes::from(json_bytes);
        PreservedSingleCall::try_from(bytes)
    }
}

fn try_from_bytes_to_vec_preserved_single_call(
    value: Bytes,
) -> Result<Vec<PreservedSingleCall>, ()> {
    let mut value = value.to_vec();
    let deserialized_json_vec: Vec<RpcCall> = simd_json::from_slice(&mut value).map_err(|_| ())?;

    deserialized_json_vec
        .into_iter()
        .map(PreservedSingleCall::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ())
}

// TODO: see if we can write a smarter serializer here - one can keep the Bytes while parsing through the JSON
impl TryFrom<Bytes> for PreservedRequest {
    type Error = ();

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let single_call = PreservedSingleCall::try_from(value.clone());
        if let Ok(single_call) = single_call {
            return Ok(PreservedRequest::Single(single_call));
        }

        if let Ok(batch_calls) = try_from_bytes_to_vec_preserved_single_call(value) {
            return Ok(PreservedRequest::Batch(batch_calls));
        }

        Err(())
    }
}

#[cfg(test)]
mod tests {
    use rpc_gateway_rpc::request::{Id, RequestParams, RpcMethodCall, Version};

    use super::*;

    #[test]
    fn test_preserved_request_deserialization_single() {
        let bytes = Bytes::from_static(b"{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"0x1\",\"false\"],\"id\":1}");

        let expected_call = RpcCall::MethodCall(RpcMethodCall {
            jsonrpc: Version::V2,
            method: "eth_getBlockByNumber".to_string(),
            params: RequestParams::Array(vec![
                simd_json::OwnedValue::String("0x1".to_string()),
                simd_json::OwnedValue::String("false".to_string()),
            ]),
            id: Id::Number(1),
        });

        let expected_preserved_request = PreservedRequest::Single(PreservedSingleCall {
            raw: bytes.clone(),
            deserialized: expected_call,
        });

        let actual_preserved_request = PreservedRequest::try_from(bytes).unwrap();

        let expected_str = simd_json::to_string(&expected_preserved_request).unwrap();
        let actual_str = simd_json::to_string(&actual_preserved_request).unwrap();

        assert_eq!(actual_str, expected_str);
    }

    #[test]
    fn test_preserved_request_deserialization_single_with_whitespace() {
        let bytes = Bytes::from_static(b"\r\t\n {\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"0x1\",\"false\"],\"id\":1}\r");
        let expected_call = RpcCall::MethodCall(RpcMethodCall {
            jsonrpc: Version::V2,
            method: "eth_getBlockByNumber".to_string(),
            params: RequestParams::Array(vec![
                simd_json::OwnedValue::String("0x1".to_string()),
                simd_json::OwnedValue::String("false".to_string()),
            ]),
            id: Id::Number(1),
        });

        let expected_preserved_request = PreservedRequest::Single(PreservedSingleCall {
            raw: bytes.clone(),
            deserialized: expected_call,
        });

        let actual_preserved_request = PreservedRequest::try_from(bytes).unwrap();
        let expected_str = simd_json::to_string(&expected_preserved_request).unwrap();
        let actual_str = simd_json::to_string(&actual_preserved_request).unwrap();

        assert_eq!(actual_str, expected_str);
    }

    #[test]
    fn test_preserved_request_deserialization_batch_empty() {
        let preserved_request = PreservedRequest::try_from(Bytes::from_static(b"[]")).unwrap();
        let expected_str = simd_json::to_string(&PreservedRequest::Batch(vec![])).unwrap();
        let actual_str = simd_json::to_string(&preserved_request).unwrap();

        assert_eq!(actual_str, expected_str);
    }

    #[test]
    fn test_preserved_request_deserialization_batch_empty_with_whitespace() {
        let preserved_request =
            PreservedRequest::try_from(Bytes::from_static(b"\r\t\n[]\r")).unwrap();
        let expected_str = simd_json::to_string(&PreservedRequest::Batch(vec![])).unwrap();
        let actual_str = simd_json::to_string(&preserved_request).unwrap();

        assert_eq!(actual_str, expected_str);
    }

    #[test]
    fn test_preserved_request_deserialization_batch_singleton() {
        let bytes = Bytes::from_static(b"[{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"0x1\",\"false\"],\"id\":1}]");
        let expected_singleton_call = RpcCall::MethodCall(RpcMethodCall {
            jsonrpc: Version::V2,
            method: "eth_getBlockByNumber".to_string(),
            params: RequestParams::Array(vec![
                simd_json::OwnedValue::String("0x1".to_string()),
                simd_json::OwnedValue::String("false".to_string()),
            ]),
            id: Id::Number(1),
        });

        let expected_bytes = Bytes::from_static(b"{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"0x1\",\"false\"],\"id\":1}");

        let expected_preserved_request = PreservedRequest::Batch(vec![PreservedSingleCall {
            raw: expected_bytes,
            deserialized: expected_singleton_call,
        }]);

        let actual_preserved_request = PreservedRequest::try_from(bytes).unwrap();
        let expected_str = simd_json::to_string(&expected_preserved_request).unwrap();
        let actual_str = simd_json::to_string(&actual_preserved_request).unwrap();

        assert_eq!(actual_str, expected_str);
    }

    #[test]
    fn test_preserved_request_deserialization_batch_singleton_with_whitespace() {
        let bytes = Bytes::from_static(b"\r\t\n[{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"0x1\",\"false\"],\"id\":1}]\r");
        let expected_singleton_call = RpcCall::MethodCall(RpcMethodCall {
            jsonrpc: Version::V2,
            method: "eth_getBlockByNumber".to_string(),
            params: RequestParams::Array(vec![
                simd_json::OwnedValue::String("0x1".to_string()),
                simd_json::OwnedValue::String("false".to_string()),
            ]),
            id: Id::Number(1),
        });

        let expected_bytes = Bytes::from_static(b"{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"0x1\",\"false\"],\"id\":1}");

        let expected_preserved_request = PreservedRequest::Batch(vec![PreservedSingleCall {
            raw: expected_bytes,
            deserialized: expected_singleton_call,
        }]);

        let actual_preserved_request = PreservedRequest::try_from(bytes).unwrap();
        let expected_str = simd_json::to_string(&expected_preserved_request).unwrap();
        let actual_str = simd_json::to_string(&actual_preserved_request).unwrap();

        assert_eq!(actual_str, expected_str);
    }

    #[test]
    fn test_preserved_request_deserialization_batch_multiple() {
        let bytes = Bytes::from_static(b"[{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"0x1\",\"false\"],\"id\":1},{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"0x1\",\"false\"],\"id\":2}]");
        let expected_call_1 = RpcCall::MethodCall(RpcMethodCall {
            jsonrpc: Version::V2,
            method: "eth_getBlockByNumber".to_string(),
            params: RequestParams::Array(vec![
                simd_json::OwnedValue::String("0x1".to_string()),
                simd_json::OwnedValue::String("false".to_string()),
            ]),
            id: Id::Number(1),
        });

        let expected_call_2 = RpcCall::MethodCall(RpcMethodCall {
            jsonrpc: Version::V2,
            method: "eth_getBlockByNumber".to_string(),
            params: RequestParams::Array(vec![
                simd_json::OwnedValue::String("0x1".to_string()),
                simd_json::OwnedValue::String("false".to_string()),
            ]),
            id: Id::Number(2),
        });

        let expected_bytes_1 = Bytes::from_static(b"{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"0x1\",\"false\"],\"id\":1}");
        let expected_bytes_2 = Bytes::from_static(b"{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"0x1\",\"false\"],\"id\":2}");

        let expected_preserved_request = PreservedRequest::Batch(vec![
            PreservedSingleCall {
                raw: expected_bytes_1,
                deserialized: expected_call_1,
            },
            PreservedSingleCall {
                raw: expected_bytes_2,
                deserialized: expected_call_2,
            },
        ]);

        let actual_preserved_request = PreservedRequest::try_from(bytes).unwrap();
        let expected_str = simd_json::to_string(&expected_preserved_request).unwrap();
        let actual_str = simd_json::to_string(&actual_preserved_request).unwrap();

        assert_eq!(actual_str, expected_str);
    }

    #[test]
    fn test_preserved_request_deserialization_batch_multiple_with_whitespace() {
        let bytes = Bytes::from_static(b"\r\t\n[{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"0x1\",\"false\"],\"id\":1},{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"0x1\",\"false\"],\"id\":2}]\r");
        let expected_call_1 = RpcCall::MethodCall(RpcMethodCall {
            jsonrpc: Version::V2,
            method: "eth_getBlockByNumber".to_string(),
            params: RequestParams::Array(vec![
                simd_json::OwnedValue::String("0x1".to_string()),
                simd_json::OwnedValue::String("false".to_string()),
            ]),
            id: Id::Number(1),
        });

        let expected_call_2 = RpcCall::MethodCall(RpcMethodCall {
            jsonrpc: Version::V2,
            method: "eth_getBlockByNumber".to_string(),
            params: RequestParams::Array(vec![
                simd_json::OwnedValue::String("0x1".to_string()),
                simd_json::OwnedValue::String("false".to_string()),
            ]),
            id: Id::Number(2),
        });

        let expected_bytes_1 = Bytes::from_static(b"{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"0x1\",\"false\"],\"id\":1}");
        let expected_bytes_2 = Bytes::from_static(b"{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"0x1\",\"false\"],\"id\":2}");

        let expected_preserved_request = PreservedRequest::Batch(vec![
            PreservedSingleCall {
                raw: expected_bytes_1,
                deserialized: expected_call_1,
            },
            PreservedSingleCall {
                raw: expected_bytes_2,
                deserialized: expected_call_2,
            },
        ]);

        let actual_preserved_request = PreservedRequest::try_from(bytes).unwrap();
        let expected_str = simd_json::to_string(&expected_preserved_request).unwrap();
        let actual_str = simd_json::to_string(&actual_preserved_request).unwrap();

        assert_eq!(actual_str, expected_str);
    }
}
