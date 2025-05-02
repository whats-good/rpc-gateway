use bytes::Bytes;
use rpc_gateway_rpc::request::{RpcCall, RpcMethodCall};
use serde_json::Value;

#[derive(Debug)]
pub enum LazyRequest {
    SingleCallOrError(SingleCallOrError),
    BatchCallOrError(BatchCallOrError),
}

#[derive(Debug)]
pub struct PreservedSingleCall {
    pub raw: Bytes,
    pub parsed: RpcCall,
}

#[derive(Debug)]
pub struct PreservedRpcMethodCall {
    pub raw: Bytes,
    pub parsed: RpcMethodCall,
}

#[derive(Debug)]
pub enum PreservedCall {
    Single(PreservedSingleCall),
    Batch(Vec<PreservedSingleCall>),
}

#[derive(Debug)]
pub struct SingleCallOrError {
    pub inner: Bytes,
}

#[derive(Debug)]
pub struct BatchCallOrError {
    pub inner: Bytes,
}

// TODO: test this.
impl TryFrom<Bytes> for LazyRequest {
    type Error = ();

    fn try_from(body: Bytes) -> Result<Self, Self::Error> {
        match body.get(0) {
            Some(0x7b) => Ok(LazyRequest::SingleCallOrError(SingleCallOrError {
                inner: body,
            })),
            Some(0x5b) => Ok(LazyRequest::BatchCallOrError(BatchCallOrError {
                inner: body,
            })),
            Some(_) => {
                // eliminate empty bytes, space, tab, newline, and carriage return
                let mut i = 0;
                while i < body.len() {
                    match body[i] {
                        0x20 | 0x09 | 0x0a | 0x0d => {
                            i += 1;
                        }
                        _ => break,
                    }
                }
                match body[i] {
                    0x7b => Ok(LazyRequest::SingleCallOrError(SingleCallOrError {
                        inner: body.slice(i..),
                    })),
                    0x5b => Ok(LazyRequest::BatchCallOrError(BatchCallOrError {
                        inner: body.slice(i..),
                    })),
                    _ => Err(()),
                }
            }
            None => Err(()),
        }
    }
}

impl TryFrom<Bytes> for PreservedSingleCall {
    type Error = ();

    fn try_from(body: Bytes) -> Result<Self, Self::Error> {
        let parsed = serde_json::from_slice::<RpcCall>(&body).map_err(|_| ())?;
        let preserved_single_call = PreservedSingleCall { raw: body, parsed };
        Ok(preserved_single_call)
    }
}

impl TryFrom<Bytes> for PreservedRpcMethodCall {
    type Error = ();

    fn try_from(body: Bytes) -> Result<Self, Self::Error> {
        let parsed = serde_json::from_slice::<RpcMethodCall>(&body).map_err(|_| ())?;
        let preserved_rpc_method_call = PreservedRpcMethodCall { raw: body, parsed };
        Ok(preserved_rpc_method_call)
    }
}

impl TryFrom<LazyRequest> for PreservedCall {
    type Error = ();
    fn try_from(lazy_request: LazyRequest) -> Result<Self, Self::Error> {
        match lazy_request {
            LazyRequest::SingleCallOrError(single_call_or_error) => {
                PreservedSingleCall::try_from(single_call_or_error.inner).map(PreservedCall::Single)
            }
            LazyRequest::BatchCallOrError(batch_call_or_error) => {
                let values = serde_json::from_slice::<Vec<Value>>(&batch_call_or_error.inner)
                    .map_err(|_| ())?;
                let preserved_batch_call_items = values
                    .into_iter()
                    .map(|v| {
                        let vec_result = serde_json::to_vec(&v).map_err(|_| ());
                        let bytes_result = vec_result.map(Bytes::from);
                        let preserved_single_call_result =
                            bytes_result.and_then(PreservedSingleCall::try_from);
                        preserved_single_call_result
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(PreservedCall::Batch(preserved_batch_call_items))
            }
        }
    }
}
