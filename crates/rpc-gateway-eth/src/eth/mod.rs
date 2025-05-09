use alloy_primitives::{Address, B256, U256};
use alloy_rpc_types::{BlockId, BlockNumberOrTag as BlockNumber};

pub trait Keyable {
    fn get_key(&self) -> String;
}

mod serde_helpers;
use serde_helpers::lenient_block_number;

type EmptyParams = Option<[u8; 0]>;

fn key_block_id(block_id: &BlockId) -> String {
    match block_id {
        BlockId::Hash(hash) => hash.to_string(),
        BlockId::Number(number) => number.to_string(),
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct AddressWithOptionalBlockId {
    pub address: Address,
    #[serde(default)]
    pub block_id: Option<BlockId>,
}

impl Keyable for AddressWithOptionalBlockId {
    fn get_key(&self) -> String {
        let block_id_string = match &self.block_id {
            Some(block_id) => key_block_id(block_id),
            None => "".to_string(),
        };
        format!("{}:{}", self.address, block_id_string)
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct EthGetLogsParams {
    pub filter: serde_json::Value,
}

impl Keyable for EthGetLogsParams {
    fn get_key(&self) -> String {
        self.filter.to_string()
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct EthGetTransactionReceiptParams {
    pub tx_hash: B256,
}

impl Keyable for EthGetTransactionReceiptParams {
    fn get_key(&self) -> String {
        self.tx_hash.to_string()
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct EthGetBlockByNumberParams {
    #[serde(deserialize_with = "lenient_block_number::lenient_block_number")]
    pub block_number: BlockNumber,
    pub full_transaction: bool,
}

impl Keyable for EthGetBlockByNumberParams {
    fn get_key(&self) -> String {
        let full_transaction_string = if self.full_transaction { "1" } else { "0" };
        format!("{}:{}", self.block_number, full_transaction_string)
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct EthCallParams {
    pub tx: serde_json::Value,
    #[serde(default)]
    pub block_id: Option<BlockId>,
    #[serde(default)]
    pub state_override: Option<serde_json::Value>,
}

impl Keyable for EthCallParams {
    fn get_key(&self) -> String {
        let block_id_string = match &self.block_id {
            Some(block_id) => key_block_id(block_id),
            None => "".to_string(),
        };
        let state_override_string = match &self.state_override {
            Some(state_override) => state_override.to_string(),
            None => "".to_string(),
        };
        format!("{}:{}:{}", self.tx, block_id_string, state_override_string)
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct EthGetStorageAtParams {
    pub address: Address,
    pub position: U256,
    #[serde(default)]
    pub block_id: Option<BlockId>,
}

impl Keyable for EthGetStorageAtParams {
    fn get_key(&self) -> String {
        let block_id_string = match &self.block_id {
            Some(block_id) => key_block_id(block_id),
            None => "".to_string(),
        };
        format!("{}:{}:{}", self.address, self.position, block_id_string)
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct EthGetBlockByHashParams {
    pub block_hash: B256,
    pub full_transaction: bool,
}

impl Keyable for EthGetBlockByHashParams {
    fn get_key(&self) -> String {
        let full_transaction_string = if self.full_transaction { "1" } else { "0" };
        format!("{}:{}", self.block_hash, full_transaction_string)
    }
}

/// Represents ethereum JSON-RPC API
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(tag = "method")]
#[expect(clippy::large_enum_variant)]
pub enum EthRequest {
    #[serde(rename = "eth_call")]
    EthCall { params: EthCallParams },

    #[serde(rename = "eth_getBalance")]
    EthGetBalance { params: AddressWithOptionalBlockId },

    #[serde(rename = "eth_blockNumber")]
    EthBlockNumber { params: EmptyParams },

    #[serde(rename = "eth_getLogs")]
    EthGetLogs { params: EthGetLogsParams },

    #[serde(rename = "eth_getTransactionReceipt")]
    EthGetTransactionReceipt {
        params: EthGetTransactionReceiptParams,
    },
    #[serde(rename = "eth_getBlockByNumber")]
    EthGetBlockByNumber { params: EthGetBlockByNumberParams },

    #[serde(rename = "eth_getCode")]
    EthGetCodeAt { params: AddressWithOptionalBlockId },

    #[serde(rename = "eth_getTransactionCount")]
    EthGetTransactionCount { params: AddressWithOptionalBlockId },

    #[serde(rename = "eth_chainId")]
    EthChainId { params: EmptyParams },

    #[serde(rename = "eth_maxPriorityFeePerGas")]
    EthMaxPriorityFeePerGas { params: EmptyParams },

    #[serde(rename = "eth_estimateGas")]
    EthEstimateGas { params: EthCallParams },

    #[serde(rename = "web3_clientVersion")]
    Web3ClientVersion { params: EmptyParams },

    #[serde(rename = "eth_networkId")]
    EthNetworkId { params: EmptyParams },

    #[serde(rename = "eth_gasPrice")]
    EthGasPrice { params: EmptyParams },

    #[serde(rename = "eth_blobBaseFee")]
    EthBlobBaseFee { params: EmptyParams },

    #[serde(rename = "eth_getStorageAt")]
    EthGetStorageAt { params: EthGetStorageAtParams },

    #[serde(rename = "eth_getBlockByHash")]
    EthGetBlockByHash { params: EthGetBlockByHashParams },
}

impl EthRequest {
    #[inline]
    fn get_key_prefix(&self) -> &'static str {
        match self {
            EthRequest::EthCall { .. } => "00",
            EthRequest::EthGetBalance { .. } => "01",
            EthRequest::EthBlockNumber { .. } => "02",
            EthRequest::EthGetLogs { .. } => "03",
            EthRequest::EthGetTransactionReceipt { .. } => "04",
            EthRequest::EthGetBlockByNumber { .. } => "05",
            EthRequest::EthGetCodeAt { .. } => "06",
            EthRequest::EthGetTransactionCount { .. } => "07",
            EthRequest::EthChainId { .. } => "08",
            EthRequest::EthMaxPriorityFeePerGas { .. } => "09",
            EthRequest::EthEstimateGas { .. } => "0A",
            EthRequest::Web3ClientVersion { .. } => "0B",
            EthRequest::EthNetworkId { .. } => "0C",
            EthRequest::EthGasPrice { .. } => "0D",
            EthRequest::EthBlobBaseFee { .. } => "0E",
            EthRequest::EthGetStorageAt { .. } => "0F",
            EthRequest::EthGetBlockByHash { .. } => "10",
        }
    }

    pub fn get_key(&self) -> String {
        let key_prefix = self.get_key_prefix();
        match self {
            EthRequest::EthCall { params } => format!("{}:{}", key_prefix, params.get_key()),
            EthRequest::EthGetBalance { params } => format!("{}:{}", key_prefix, params.get_key()),
            EthRequest::EthBlockNumber { .. } => key_prefix.to_string(),
            EthRequest::EthGetLogs { params } => format!("{}:{}", key_prefix, params.get_key()),
            EthRequest::EthGetTransactionReceipt { params } => {
                format!("{}:{}", key_prefix, params.get_key())
            }
            EthRequest::EthGetBlockByNumber { params } => {
                format!("{}:{}", key_prefix, params.get_key())
            }
            EthRequest::EthGetCodeAt { params } => format!("{}:{}", key_prefix, params.get_key()),
            EthRequest::EthGetTransactionCount { params } => {
                format!("{}:{}", key_prefix, params.get_key())
            }
            EthRequest::EthChainId { .. } => key_prefix.to_string(),
            EthRequest::EthMaxPriorityFeePerGas { .. } => key_prefix.to_string(),
            EthRequest::EthEstimateGas { params } => format!("{}:{}", key_prefix, params.get_key()),
            EthRequest::Web3ClientVersion { .. } => key_prefix.to_string(),
            EthRequest::EthNetworkId { .. } => key_prefix.to_string(),
            EthRequest::EthGasPrice { .. } => key_prefix.to_string(),
            EthRequest::EthBlobBaseFee { .. } => key_prefix.to_string(),
            EthRequest::EthGetStorageAt { params } => {
                format!("{}:{}", key_prefix, params.get_key())
            }
            EthRequest::EthGetBlockByHash { params } => {
                format!("{}:{}", key_prefix, params.get_key())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_eth_block_number_empty_params() {
        let string = r#"{"method":"eth_blockNumber","params":[],"id":1}"#;
        let request: EthRequest = serde_json::from_str(string).unwrap();
        matches!(request, EthRequest::EthBlockNumber { params: None });
    }

    #[test]
    fn test_eth_block_number_omitted_params() {
        let string = r#"{"method":"eth_blockNumber","id":1}"#;
        let request: EthRequest = serde_json::from_str(string).unwrap();
        matches!(request, EthRequest::EthBlockNumber { params: None });
    }

    #[test]
    fn test_eth_get_balance_all_params() {
        let string = r#"{"method":"eth_getBalance","params":["0x0000000000000000000000000000000000000000","latest"],"id":1}"#;
        let request: EthRequest = serde_json::from_str(string).unwrap();
        if let EthRequest::EthGetBalance { params } = request {
            assert_eq!(params.address, Address::ZERO);
            assert_eq!(params.block_id, Some(BlockId::latest()));
        } else {
            panic!("expected EthRequest::EthGetBalance");
        }
    }

    #[test]
    fn test_eth_get_balance_block_id_omitted() {
        let string = r#"{"method":"eth_getBalance","params":["0x0000000000000000000000000000000000000000"],"id":1}"#;
        let request: EthRequest = serde_json::from_str(string).unwrap();
        if let EthRequest::EthGetBalance { params } = request {
            assert_eq!(params.address, Address::ZERO);
            assert_eq!(params.block_id, None);
        } else {
            panic!("expected EthRequest::EthGetBalance");
        }
    }

    #[test]
    fn test_eth_get_balance_all_params_omitted() {
        let string = r#"{"method":"eth_getBalance","id":1}"#;
        let request: Result<EthRequest, _> = serde_json::from_str(string);
        matches!(request, Err(_));
    }

    #[test]
    fn test_eth_get_balance_empty_params() {
        let string = r#"{"method":"eth_getBalance","params":[],"id":1}"#;
        let request: Result<EthRequest, _> = serde_json::from_str(string);
        matches!(request, Err(_));
    }

    #[test]
    fn test_eth_get_transaction_receipt() {
        let string = r#"{"method":"eth_getTransactionReceipt","params":["0x0000000000000000000000000000000000000000000000000000000000000001"],"id":1}"#;
        let request: EthRequest = serde_json::from_str(string).unwrap();
        if let EthRequest::EthGetTransactionReceipt { params } = request {
            assert_eq!(
                params.tx_hash,
                B256::from_str(
                    "0x0000000000000000000000000000000000000000000000000000000000000001"
                )
                .unwrap()
            );
        } else {
            panic!("expected EthRequest::EthGetTransactionReceipt");
        }
    }

    #[test]
    fn test_eth_get_block_by_number_latest_true() {
        let string = r#"{"method":"eth_getBlockByNumber","params":["latest", true],"id":1}"#;
        let request: EthRequest = serde_json::from_str(string).unwrap();
        if let EthRequest::EthGetBlockByNumber { params } = request {
            assert_eq!(params.block_number, BlockNumber::Latest);
            assert_eq!(params.full_transaction, true);
        } else {
            panic!("expected EthRequest::EthGetBlockByNumber");
        }
    }

    #[test]
    fn test_eth_get_block_by_number_actual_block_number_latest_false() {
        let string = r#"{"method":"eth_getBlockByNumber","params":["0x1", false],"id":1}"#;
        let request: EthRequest = serde_json::from_str(string).unwrap();
        if let EthRequest::EthGetBlockByNumber { params } = request {
            assert_eq!(params.block_number, BlockNumber::Number(1));
            assert_eq!(params.full_transaction, false);
        } else {
            panic!("expected EthRequest::EthGetBlockByNumber");
        }
    }

    #[test]
    fn test_eth_get_code_at_all_params() {
        let string = r#"{"method":"eth_getCode","params":["0x0000000000000000000000000000000000000000", "latest"],"id":1}"#;
        let request: EthRequest = serde_json::from_str(string).unwrap();
        if let EthRequest::EthGetCodeAt { params } = request {
            assert_eq!(params.address, Address::ZERO);
            assert_eq!(params.block_id, Some(BlockId::latest()));
        } else {
            panic!("expected EthRequest::EthGetCodeAt");
        }
    }

    #[test]
    fn test_eth_get_code_at_block_id_omitted() {
        let string = r#"{"method":"eth_getCode","params":["0x0000000000000000000000000000000000000000"],"id":1}"#;
        let request: EthRequest = serde_json::from_str(string).unwrap();
        if let EthRequest::EthGetCodeAt { params } = request {
            assert_eq!(params.address, Address::ZERO);
            assert_eq!(params.block_id, None);
        } else {
            panic!("expected EthRequest::EthGetCodeAt");
        }
    }

    #[test]
    fn test_eth_get_storage_at_all_params() {
        let string = r#"{"method":"eth_getStorageAt","params":["0x0000000000000000000000000000000000000000", "0x1", "latest"],"id":1}"#;
        let request: EthRequest = serde_json::from_str(string).unwrap();
        if let EthRequest::EthGetStorageAt { params } = request {
            assert_eq!(params.address, Address::ZERO);
            assert_eq!(params.position, U256::from(1));
            assert_eq!(params.block_id, Some(BlockId::latest()));
        } else {
            panic!("expected EthRequest::EthGetStorageAt");
        }
    }

    #[test]
    fn test_eth_get_storage_at_block_id_omitted() {
        let string = r#"{"method":"eth_getStorageAt","params":["0x0000000000000000000000000000000000000000", "0x1"],"id":1}"#;
        let request: EthRequest = serde_json::from_str(string).unwrap();
        if let EthRequest::EthGetStorageAt { params } = request {
            assert_eq!(params.address, Address::ZERO);
            assert_eq!(params.position, U256::from(1));
            assert_eq!(params.block_id, None);
        } else {
            panic!("expected EthRequest::EthGetStorageAt");
        }
    }

    #[test]
    fn test_eth_get_block_by_hash_all_params() {
        let string = r#"{"method":"eth_getBlockByHash","params":["0x0000000000000000000000000000000000000000000000000000000000000001", true],"id":1}"#;
        let request: EthRequest = serde_json::from_str(string).unwrap();
        if let EthRequest::EthGetBlockByHash { params } = request {
            assert_eq!(
                params.block_hash,
                B256::from_str(
                    "0x0000000000000000000000000000000000000000000000000000000000000001"
                )
                .unwrap()
            );
            assert_eq!(params.full_transaction, true);
        } else {
            panic!("expected EthRequest::EthGetBlockByHash");
        }
    }
}
