use alloy_primitives::{Address, B256, Bytes, TxHash, U256};
use alloy_rpc_types::{
    BlockId, BlockNumberOrTag as BlockNumber, Filter, Index, request::TransactionRequest,
    state::StateOverride,
};
use alloy_serde::WithOtherFields;

mod serde_helpers;
use self::serde_helpers::*;

/// Wrapper type that ensures the type is named `params`
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Params<T: Default> {
    #[serde(default)]
    pub params: T,
}

/// Represents ethereum JSON-RPC API
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "method", content = "params")]
#[expect(clippy::large_enum_variant)]
pub enum EthRequest {
    #[serde(rename = "eth_call")]
    EthCall(
        WithOtherFields<TransactionRequest>,
        #[serde(default)] Option<BlockId>,
        #[serde(default)] Option<StateOverride>,
    ),

    #[serde(rename = "eth_getBalance")]
    EthGetBalance(Address, Option<BlockId>),

    #[serde(rename = "eth_blockNumber", with = "empty_params")]
    EthBlockNumber(()),

    #[serde(rename = "eth_getLogs", with = "sequence")]
    EthGetLogs(Filter),

    #[serde(rename = "eth_getTransactionReceipt", with = "sequence")]
    EthGetTransactionReceipt(B256),

    #[serde(rename = "eth_getBlockByNumber")]
    EthGetBlockByNumber(
        #[serde(deserialize_with = "lenient_block_number::lenient_block_number")] BlockNumber,
        bool,
    ),

    #[serde(rename = "eth_getCode")]
    EthGetCodeAt(Address, Option<BlockId>),

    #[serde(rename = "eth_getTransactionCount")]
    EthGetTransactionCount(Address, Option<BlockId>),

    #[serde(rename = "eth_chainId", with = "empty_params")]
    EthChainId(()),

    #[serde(rename = "eth_maxPriorityFeePerGas", with = "empty_params")]
    EthMaxPriorityFeePerGas(()),

    #[serde(rename = "eth_estimateGas")]
    EthEstimateGas(
        WithOtherFields<TransactionRequest>,
        #[serde(default)] Option<BlockId>,
        #[serde(default)] Option<StateOverride>,
    ),

    #[serde(rename = "web3_clientVersion", with = "empty_params")]
    Web3ClientVersion(()),

    #[serde(rename = "eth_networkId", alias = "net_version", with = "empty_params")]
    EthNetworkId(()),

    #[serde(rename = "eth_gasPrice", with = "empty_params")]
    EthGasPrice(()),

    #[serde(rename = "eth_blobBaseFee", with = "empty_params")]
    EthBlobBaseFee(()),

    #[serde(rename = "eth_getStorageAt")]
    EthGetStorageAt(Address, U256, Option<BlockId>),

    #[serde(rename = "eth_getBlockByHash")]
    EthGetBlockByHash(B256, bool),
    // #[serde(rename = "web3_sha3", with = "sequence")]
    // Web3Sha3(Bytes),

    // #[serde(rename = "eth_getAccount")]
    // EthGetAccount(Address, Option<BlockId>),

    // #[serde(rename = "eth_getBlockTransactionCountByHash", with = "sequence")]
    // EthGetTransactionCountByHash(B256),

    // #[serde(
    //     rename = "eth_getBlockTransactionCountByNumber",
    //     deserialize_with = "lenient_block_number::lenient_block_number_seq"
    // )]
    // EthGetTransactionCountByNumber(BlockNumber),

    // #[serde(rename = "eth_getUncleCountByBlockHash", with = "sequence")]
    // EthGetUnclesCountByHash(B256),

    // #[serde(
    //     rename = "eth_getUncleCountByBlockNumber",
    //     deserialize_with = "lenient_block_number::lenient_block_number_seq"
    // )]
    // EthGetUnclesCountByNumber(BlockNumber),

    // #[serde(rename = "eth_getProof")]
    // EthGetProof(Address, Vec<B256>, Option<BlockId>),

    // #[serde(rename = "eth_getTransactionByHash", with = "sequence")]
    // EthGetTransactionByHash(TxHash),

    // #[serde(rename = "eth_getTransactionByBlockHashAndIndex")]
    // EthGetTransactionByBlockHashAndIndex(TxHash, Index),

    // #[serde(rename = "eth_getTransactionByBlockNumberAndIndex")]
    // EthGetTransactionByBlockNumberAndIndex(BlockNumber, Index),

    // #[serde(rename = "eth_getRawTransactionByHash", with = "sequence")]
    // EthGetRawTransactionByHash(TxHash),

    // #[serde(rename = "eth_getRawTransactionByBlockHashAndIndex")]
    // EthGetRawTransactionByBlockHashAndIndex(TxHash, Index),

    // #[serde(rename = "eth_getRawTransactionByBlockNumberAndIndex")]
    // EthGetRawTransactionByBlockNumberAndIndex(BlockNumber, Index),

    // #[serde(rename = "eth_getBlockReceipts", with = "sequence")]
    // EthGetBlockReceipts(BlockId),

    // #[serde(rename = "eth_getUncleByBlockHashAndIndex")]
    // EthGetUncleByBlockHashAndIndex(B256, Index),

    // #[serde(rename = "eth_getUncleByBlockNumberAndIndex")]
    // EthGetUncleByBlockNumberAndIndex(
    //     #[serde(deserialize_with = "lenient_block_number::lenient_block_number")] BlockNumber,
    //     Index,
    // ),

    // /// Creates a filter object, based on filter options, to notify when the state changes (logs).
    // #[serde(rename = "eth_newFilter", with = "sequence")]
    // EthNewFilter(Filter),

    // /// Polling method for a filter, which returns an array of logs which occurred since last poll.
    // #[serde(rename = "eth_getFilterChanges", with = "sequence")]
    // EthGetFilterChanges(String),

    // /// Returns an array of all logs matching filter with given id.
    // #[serde(rename = "eth_getFilterLogs", with = "sequence")]
    // EthGetFilterLogs(String),

    // /// Removes the filter, returns true if the filter was installed
    // #[serde(rename = "eth_uninstallFilter", with = "sequence")]
    // EthUninstallFilter(String),

    // #[serde(rename = "eth_feeHistory")]
    // EthFeeHistory(
    //     #[serde(deserialize_with = "deserialize_number")] U256,
    //     BlockNumber,
    //     #[serde(default)] Vec<f64>,
    // ),
}
