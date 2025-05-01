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
    #[serde(rename = "web3_clientVersion", with = "empty_params")]
    Web3ClientVersion(()),

    #[serde(rename = "web3_sha3", with = "sequence")]
    Web3Sha3(Bytes),

    #[serde(rename = "eth_chainId", with = "empty_params")]
    EthChainId(()),

    #[serde(rename = "eth_networkId", alias = "net_version", with = "empty_params")]
    EthNetworkId(()),

    #[serde(rename = "eth_gasPrice", with = "empty_params")]
    EthGasPrice(()),

    #[serde(rename = "eth_maxPriorityFeePerGas", with = "empty_params")]
    EthMaxPriorityFeePerGas(()),

    #[serde(rename = "eth_blobBaseFee", with = "empty_params")]
    EthBlobBaseFee(()),

    #[serde(rename = "eth_blockNumber", with = "empty_params")]
    EthBlockNumber(()),

    #[serde(rename = "eth_getBalance")]
    EthGetBalance(Address, Option<BlockId>),

    #[serde(rename = "eth_getAccount")]
    EthGetAccount(Address, Option<BlockId>),

    #[serde(rename = "eth_getStorageAt")]
    EthGetStorageAt(Address, U256, Option<BlockId>),

    #[serde(rename = "eth_getBlockByHash")]
    EthGetBlockByHash(B256, bool),

    #[serde(rename = "eth_getBlockByNumber")]
    EthGetBlockByNumber(
        #[serde(deserialize_with = "lenient_block_number::lenient_block_number")] BlockNumber,
        bool,
    ),

    #[serde(rename = "eth_getTransactionCount")]
    EthGetTransactionCount(Address, Option<BlockId>),

    #[serde(rename = "eth_getBlockTransactionCountByHash", with = "sequence")]
    EthGetTransactionCountByHash(B256),

    #[serde(
        rename = "eth_getBlockTransactionCountByNumber",
        deserialize_with = "lenient_block_number::lenient_block_number_seq"
    )]
    EthGetTransactionCountByNumber(BlockNumber),

    #[serde(rename = "eth_getUncleCountByBlockHash", with = "sequence")]
    EthGetUnclesCountByHash(B256),

    #[serde(
        rename = "eth_getUncleCountByBlockNumber",
        deserialize_with = "lenient_block_number::lenient_block_number_seq"
    )]
    EthGetUnclesCountByNumber(BlockNumber),

    #[serde(rename = "eth_getCode")]
    EthGetCodeAt(Address, Option<BlockId>),

    #[serde(rename = "eth_getProof")]
    EthGetProof(Address, Vec<B256>, Option<BlockId>),

    #[serde(rename = "eth_call")]
    EthCall(
        WithOtherFields<TransactionRequest>,
        #[serde(default)] Option<BlockId>,
        #[serde(default)] Option<StateOverride>,
    ),

    #[serde(rename = "eth_estimateGas")]
    EthEstimateGas(
        WithOtherFields<TransactionRequest>,
        #[serde(default)] Option<BlockId>,
        #[serde(default)] Option<StateOverride>,
    ),

    #[serde(rename = "eth_getTransactionByHash", with = "sequence")]
    EthGetTransactionByHash(TxHash),

    #[serde(rename = "eth_getTransactionByBlockHashAndIndex")]
    EthGetTransactionByBlockHashAndIndex(TxHash, Index),

    #[serde(rename = "eth_getTransactionByBlockNumberAndIndex")]
    EthGetTransactionByBlockNumberAndIndex(BlockNumber, Index),

    #[serde(rename = "eth_getRawTransactionByHash", with = "sequence")]
    EthGetRawTransactionByHash(TxHash),

    #[serde(rename = "eth_getRawTransactionByBlockHashAndIndex")]
    EthGetRawTransactionByBlockHashAndIndex(TxHash, Index),

    #[serde(rename = "eth_getRawTransactionByBlockNumberAndIndex")]
    EthGetRawTransactionByBlockNumberAndIndex(BlockNumber, Index),

    #[serde(rename = "eth_getTransactionReceipt", with = "sequence")]
    EthGetTransactionReceipt(B256),

    #[serde(rename = "eth_getBlockReceipts", with = "sequence")]
    EthGetBlockReceipts(BlockId),

    #[serde(rename = "eth_getUncleByBlockHashAndIndex")]
    EthGetUncleByBlockHashAndIndex(B256, Index),

    #[serde(rename = "eth_getUncleByBlockNumberAndIndex")]
    EthGetUncleByBlockNumberAndIndex(
        #[serde(deserialize_with = "lenient_block_number::lenient_block_number")] BlockNumber,
        Index,
    ),

    #[serde(rename = "eth_getLogs", with = "sequence")]
    EthGetLogs(Filter),

    /// Creates a filter object, based on filter options, to notify when the state changes (logs).
    #[serde(rename = "eth_newFilter", with = "sequence")]
    EthNewFilter(Filter),

    /// Polling method for a filter, which returns an array of logs which occurred since last poll.
    #[serde(rename = "eth_getFilterChanges", with = "sequence")]
    EthGetFilterChanges(String),

    /// Creates a filter in the node, to notify when a new block arrives.
    /// To check if the state has changed, call `eth_getFilterChanges`.
    #[serde(rename = "eth_newBlockFilter", with = "empty_params")]
    EthNewBlockFilter(()),

    /// Creates a filter in the node, to notify when new pending transactions arrive.
    /// To check if the state has changed, call `eth_getFilterChanges`.
    #[serde(rename = "eth_newPendingTransactionFilter", with = "empty_params")]
    EthNewPendingTransactionFilter(()),

    /// Returns an array of all logs matching filter with given id.
    #[serde(rename = "eth_getFilterLogs", with = "sequence")]
    EthGetFilterLogs(String),

    /// Removes the filter, returns true if the filter was installed
    #[serde(rename = "eth_uninstallFilter", with = "sequence")]
    EthUninstallFilter(String),

    #[serde(rename = "eth_feeHistory")]
    EthFeeHistory(
        #[serde(deserialize_with = "deserialize_number")] U256,
        BlockNumber,
        #[serde(default)] Vec<f64>,
    ),
}

// /// Represents ethereum JSON-RPC API
// #[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
// #[serde(tag = "method", content = "params")]
// pub enum EthPubSub {
//     /// Subscribe to an eth subscription
//     #[serde(rename = "eth_subscribe")]
//     EthSubscribe(SubscriptionKind, #[serde(default)] Box<SubscriptionParams>),

//     /// Unsubscribe from an eth subscription
//     #[serde(rename = "eth_unsubscribe", with = "sequence")]
//     EthUnSubscribe(SubscriptionId),
// }

// /// Container type for either a request or a pub sub
// #[derive(Clone, Debug, serde::Deserialize)]
// #[serde(untagged)]
// pub enum EthRpcCall {
//     Request(Box<EthRequest>),
//     PubSub(EthPubSub),
// }

// fn hash_block_id<H: Hasher>(block_id: &BlockId, state: &mut H) {
//     match block_id {
//         BlockId::Hash(hash) => hash.block_hash.hash(state),
//         BlockId::Number(number) => number.hash(state),
//     }
// }

// fn hash_typed_data<H: Hasher>(typed_data: &TypedData, state: &mut H) {
//     match typed_data.hash_struct() {
//         Ok(bytes) => bytes.hash(state),
//         Err(_) => {
//             // TODO: double check this part
//             "error".hash(state);
//         }
//     }
// }

// fn hash_account_override<H: Hasher>(account_override: &AccountOverride, state: &mut H) {
//     match serde_json::to_string(&account_override) {
//         Ok(account_override_str) => account_override_str.hash(state),
//         Err(_) => {
//             // TODO: double check this part
//             "error".hash(state);
//         }
//     }
// }

// fn hash_trace_filter<H: Hasher>(trace_filter: &TraceFilter, state: &mut H) {
//     match serde_json::to_string(&trace_filter) {
//         Ok(trace_filter_str) => trace_filter_str.hash(state),
//         Err(_) => {
//             // TODO: double check this part
//             "error".hash(state);
//         }
//     }
// }

// fn hash_geth_debug_tracing_options<H: Hasher>(
//     geth_debug_tracing_options: &GethDebugTracingOptions,
//     state: &mut H,
// ) {
//     match serde_json::to_string(&geth_debug_tracing_options) {
//         Ok(geth_debug_tracing_options_str) => geth_debug_tracing_options_str.hash(state),
//         Err(_) => {
//             // TODO: double check this part
//             "error".hash(state);
//         }
//     }
// }

// fn hash_geth_debug_tracing_call_options<H: Hasher>(
//     geth_debug_tracing_call_options: &GethDebugTracingCallOptions,
//     state: &mut H,
// ) {
//     match serde_json::to_string(&geth_debug_tracing_call_options) {
//         Ok(geth_debug_tracing_call_options_str) => geth_debug_tracing_call_options_str.hash(state),
//         Err(_) => {
//             // TODO: double check this part
//             "error".hash(state);
//         }
//     }
// }

// fn hash_simulate_payload<H: Hasher>(simulate_payload: &SimulatePayload, state: &mut H) {
//     match serde_json::to_string(&simulate_payload) {
//         Ok(simulate_payload_str) => simulate_payload_str.hash(state),
//         Err(_) => {
//             // TODO: double check this part
//             "error".hash(state);
//         }
//     }
// }

// impl Hash for EthRequest {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         match self {
//             EthRequest::Web3ClientVersion(_) => {
//                 "web3_clientVersion".hash(state);
//             }
//             EthRequest::Web3Sha3(bytes) => {
//                 "web3_sha3".hash(state);
//                 bytes.hash(state);
//             }
//             EthRequest::EthChainId(_) => {
//                 "eth_chainId".hash(state);
//             }
//             EthRequest::EthNetworkId(_) => {
//                 "eth_networkId".hash(state);
//             }
//             EthRequest::NetListening(_) => {
//                 "net_listening".hash(state);
//             }
//             EthRequest::EthGasPrice(_) => {
//                 "eth_gasPrice".hash(state);
//             }
//             EthRequest::EthMaxPriorityFeePerGas(_) => {
//                 "eth_maxPriorityFeePerGas".hash(state);
//             }
//             EthRequest::EthBlobBaseFee(_) => {
//                 "eth_blobBaseFee".hash(state);
//             }
//             EthRequest::EthAccounts(_) => {
//                 "eth_accounts".hash(state);
//             }
//             EthRequest::EthBlockNumber(_) => {
//                 "eth_blockNumber".hash(state);
//             }
//             EthRequest::EthGetBalance(address, block_id) => {
//                 "eth_getBalance".hash(state);
//                 address.hash(state);
//                 if let Some(block_id) = block_id {
//                     hash_block_id(block_id, state);
//                 }
//             }
//             EthRequest::EthGetAccount(address, block_id) => {
//                 "eth_getAccount".hash(state);
//                 address.hash(state);
//                 if let Some(block_id) = block_id {
//                     hash_block_id(block_id, state);
//                 }
//             }
//             EthRequest::EthGetBlockByNumber(block_number_or_tag, _) => {
//                 "eth_getBlockByNumber".hash(state);
//                 block_number_or_tag.hash(state);
//             }
//             EthRequest::EthGetTransactionCount(address, block_id) => {
//                 "eth_getTransactionCount".hash(state);
//                 address.hash(state);
//                 if let Some(block_id) = block_id {
//                     hash_block_id(block_id, state);
//                 }
//             }
//             EthRequest::EthGetTransactionCountByHash(fixed_bytes) => {
//                 "eth_getTransactionCountByHash".hash(state);
//                 fixed_bytes.hash(state);
//             }
//             EthRequest::EthGetTransactionCountByNumber(block_number_or_tag) => {
//                 "eth_getTransactionCountByNumber".hash(state);
//                 block_number_or_tag.hash(state);
//             }
//             EthRequest::EthGetUnclesCountByNumber(block_number_or_tag) => {
//                 "eth_getUnclesCountByNumber".hash(state);
//                 block_number_or_tag.hash(state);
//             }
//             EthRequest::EthGetCodeAt(address, block_id) => {
//                 "eth_getCodeAt".hash(state);
//                 address.hash(state);
//                 if let Some(block_id) = block_id {
//                     hash_block_id(block_id, state);
//                 }
//             }
//             EthRequest::EthGetProof(address, items, block_id) => {
//                 "eth_getProof".hash(state);
//                 address.hash(state);
//                 items.hash(state);
//                 if let Some(block_id) = block_id {
//                     hash_block_id(block_id, state);
//                 }
//             }
//             EthRequest::EthSign(address, bytes) => {
//                 "eth_sign".hash(state);
//                 address.hash(state);
//                 bytes.hash(state);
//             }
//             EthRequest::PersonalSign(bytes, address) => {
//                 "personal_sign".hash(state);
//                 bytes.hash(state);
//                 address.hash(state);
//             }
//             EthRequest::EthSignTransaction(_) => {
//                 "eth_signTransaction".hash(state);
//             }
//             EthRequest::EthSignTypedData(address, value) => {
//                 "eth_signTypedData".hash(state);
//                 address.hash(state);
//                 value.hash(state);
//             }
//             EthRequest::EthSignTypedDataV3(address, value) => {
//                 "eth_signTypedDataV3".hash(state);
//                 address.hash(state);
//                 value.hash(state);
//             }
//             EthRequest::EthSignTypedDataV4(address, typed_data) => {
//                 "eth_signTypedDataV4".hash(state);
//                 address.hash(state);
//                 hash_typed_data(typed_data, state);
//             }
//             EthRequest::EthSendTransaction(tx) => {
//                 "eth_sendTransaction".hash(state);
//                 tx.hash(state);
//             }
//             EthRequest::EthSendRawTransaction(bytes) => {
//                 "eth_sendRawTransaction".hash(state);
//                 bytes.hash(state);
//             }
//             EthRequest::EthCall(_, block_id, hash_map) => {
//                 "eth_call".hash(state);
//                 if let Some(block_id) = block_id {
//                     hash_block_id(block_id, state);
//                 }
//                 if let Some(hash_map) = hash_map {
//                     for (key, value) in hash_map {
//                         key.hash(state);
//                         hash_account_override(value, state);
//                     }
//                 }
//             }
//             EthRequest::EthSimulateV1(simulate_payload, block_id) => {
//                 "eth_simulateV1".hash(state);
//                 hash_simulate_payload(simulate_payload, state);
//                 if let Some(block_id) = block_id {
//                     hash_block_id(block_id, state);
//                 }
//             }
//             EthRequest::EthCreateAccessList(_, block_id) => {
//                 "eth_createAccessList".hash(state);
//                 if let Some(block_id) = block_id {
//                     hash_block_id(block_id, state);
//                 }
//             }
//             EthRequest::EthEstimateGas(_, block_id, hash_map) => {
//                 "eth_estimateGas".hash(state);
//                 if let Some(block_id) = block_id {
//                     hash_block_id(block_id, state);
//                 }
//                 if let Some(hash_map) = hash_map {
//                     for (key, value) in hash_map {
//                         key.hash(state);
//                         hash_account_override(value, state);
//                     }
//                 }
//             }
//             EthRequest::EthGetTransactionByHash(fixed_bytes) => {
//                 "eth_getTransactionByHash".hash(state);
//                 fixed_bytes.hash(state);
//             }
//             EthRequest::EthGetTransactionByBlockHashAndIndex(fixed_bytes, index) => {
//                 "eth_getTransactionByBlockHashAndIndex".hash(state);
//                 fixed_bytes.hash(state);
//                 index.hash(state);
//             }
//             EthRequest::EthGetTransactionByBlockNumberAndIndex(block_number_or_tag, index) => {
//                 "eth_getTransactionByBlockNumberAndIndex".hash(state);
//                 block_number_or_tag.hash(state);
//                 index.hash(state);
//             }
//             EthRequest::EthGetRawTransactionByHash(fixed_bytes) => {
//                 "eth_getRawTransactionByHash".hash(state);
//                 fixed_bytes.hash(state);
//             }
//             EthRequest::EthGetRawTransactionByBlockHashAndIndex(fixed_bytes, index) => {
//                 "eth_getRawTransactionByBlockHashAndIndex".hash(state);
//                 fixed_bytes.hash(state);
//                 index.hash(state);
//             }
//             EthRequest::EthGetRawTransactionByBlockNumberAndIndex(block_number_or_tag, index) => {
//                 "eth_getRawTransactionByBlockNumberAndIndex".hash(state);
//                 block_number_or_tag.hash(state);
//                 index.hash(state);
//             }
//             EthRequest::EthGetTransactionReceipt(fixed_bytes) => {
//                 "eth_getTransactionReceipt".hash(state);
//                 fixed_bytes.hash(state);
//             }
//             EthRequest::EthGetBlockReceipts(block_id) => {
//                 "eth_getBlockReceipts".hash(state);
//                 hash_block_id(block_id, state);
//             }
//             EthRequest::EthGetUncleByBlockHashAndIndex(fixed_bytes, index) => {
//                 "eth_getUncleByBlockHashAndIndex".hash(state);
//                 fixed_bytes.hash(state);
//                 index.hash(state);
//             }
//             EthRequest::EthGetUncleByBlockNumberAndIndex(block_number_or_tag, index) => {
//                 "eth_getUncleByBlockNumberAndIndex".hash(state);
//                 block_number_or_tag.hash(state);
//                 index.hash(state);
//             }
//             EthRequest::EthGetLogs(filter) => {
//                 "eth_getLogs".hash(state);
//                 filter.hash(state);
//             }
//             EthRequest::EthNewFilter(filter) => {
//                 "eth_newFilter".hash(state);
//                 filter.hash(state);
//             }
//             EthRequest::EthGetFilterChanges(filter) => {
//                 "eth_getFilterChanges".hash(state);
//                 filter.hash(state);
//             }
//             EthRequest::EthNewBlockFilter(_) => {
//                 "eth_newBlockFilter".hash(state);
//             }
//             EthRequest::EthNewPendingTransactionFilter(_) => {
//                 "eth_newPendingTransactionFilter".hash(state);
//             }
//             EthRequest::EthGetFilterLogs(filter) => {
//                 "eth_getFilterLogs".hash(state);
//                 filter.hash(state);
//             }
//             EthRequest::EthUninstallFilter(filter) => {
//                 "eth_uninstallFilter".hash(state);
//                 filter.hash(state);
//             }
//             EthRequest::EthGetWork(_) => {
//                 "eth_getWork".hash(state);
//             }
//             EthRequest::EthSubmitWork(fixed_bytes, fixed_bytes1, fixed_bytes2) => {
//                 "eth_submitWork".hash(state);
//                 fixed_bytes.hash(state);
//                 fixed_bytes1.hash(state);
//                 fixed_bytes2.hash(state);
//             }
//             EthRequest::EthSubmitHashRate(uint, fixed_bytes) => {
//                 "eth_submitHashRate".hash(state);
//                 uint.hash(state);
//                 fixed_bytes.hash(state);
//             }
//             EthRequest::EthFeeHistory(uint, block_number_or_tag, items) => {
//                 "eth_feeHistory".hash(state);
//                 uint.hash(state);
//                 block_number_or_tag.hash(state);
//                 for item in items {
//                     item.to_bits().hash(state);
//                 }
//             }
//             EthRequest::EthSyncing(_) => {
//                 "eth_syncing".hash(state);
//             }
//             EthRequest::DebugGetRawTransaction(fixed_bytes) => {
//                 "debug_getRawTransaction".hash(state);
//                 fixed_bytes.hash(state);
//             }
//             EthRequest::TraceTransaction(fixed_bytes) => {
//                 "trace_transaction".hash(state);
//                 fixed_bytes.hash(state);
//             }
//             EthRequest::TraceBlock(block_number_or_tag) => {
//                 "trace_block".hash(state);
//                 block_number_or_tag.hash(state);
//             }
//             EthRequest::TraceFilter(trace_filter) => {
//                 "trace_filter".hash(state);
//                 hash_trace_filter(trace_filter, state);
//             }
//             EthRequest::DebugTraceTransaction(fixed_bytes, geth_debug_tracing_options) => {
//                 "debug_traceTransaction".hash(state);
//                 fixed_bytes.hash(state);
//                 hash_geth_debug_tracing_options(geth_debug_tracing_options, state);
//             }
//             EthRequest::DebugTraceCall(_, block_id, geth_debug_tracing_call_options) => {
//                 "debug_traceCall".hash(state);
//                 if let Some(block_id) = block_id {
//                     hash_block_id(block_id, state);
//                 }
//                 hash_geth_debug_tracing_call_options(geth_debug_tracing_call_options, state);
//             }

//             EthRequest::EthGetStorageAt(address, uint, block_id) => {
//                 "eth_getStorageAt".hash(state);
//                 address.hash(state);
//                 uint.hash(state);
//                 if let Some(block_id) = block_id {
//                     hash_block_id(block_id, state);
//                 }
//             }
//             EthRequest::EthGetBlockByHash(fixed_bytes, _) => {
//                 "eth_getBlockByHash".hash(state);
//                 fixed_bytes.hash(state);
//             }
//             EthRequest::EthGetUnclesCountByHash(fixed_bytes) => {
//                 "eth_getUnclesCountByHash".hash(state);
//                 fixed_bytes.hash(state);
//             }
//             EthRequest::EthSendUnsignedTransaction(tx) => {
//                 "eth_sendUnsignedTransaction".hash(state);
//                 tx.hash(state);
//             }
//             EthRequest::TxPoolStatus(_) => {
//                 "txpool_status".hash(state);
//             }
//             EthRequest::TxPoolInspect(_) => {
//                 "txpool_inspect".hash(state);
//             }
//             EthRequest::TxPoolContent(_) => {
//                 "txpool_content".hash(state);
//             }
//             EthRequest::ErigonGetHeaderByNumber(block_number_or_tag) => {
//                 block_number_or_tag.hash(state);
//             }

//             // ***** OTS *****
//             EthRequest::OtsGetApiLevel(_) => {
//                 "ots_getApiLevel".hash(state);
//             }
//             EthRequest::OtsGetInternalOperations(fixed_bytes) => {
//                 "ots_getInternalOperations".hash(state);
//                 fixed_bytes.hash(state);
//             }
//             EthRequest::OtsHasCode(address, block_number_or_tag) => {
//                 "ots_hasCode".hash(state);
//                 address.hash(state);
//                 block_number_or_tag.hash(state);
//             }
//             EthRequest::OtsTraceTransaction(fixed_bytes) => {
//                 "ots_traceTransaction".hash(state);
//                 fixed_bytes.hash(state);
//             }
//             EthRequest::OtsGetTransactionError(fixed_bytes) => {
//                 "ots_getTransactionError".hash(state);
//                 fixed_bytes.hash(state);
//             }
//             EthRequest::OtsGetBlockDetails(block_number_or_tag) => {
//                 "ots_getBlockDetails".hash(state);
//                 block_number_or_tag.hash(state);
//             }
//             EthRequest::OtsGetBlockDetailsByHash(fixed_bytes) => {
//                 "ots_getBlockDetailsByHash".hash(state);
//                 fixed_bytes.hash(state);
//             }
//             EthRequest::OtsGetBlockTransactions(block_number_or_tag, page, page_size) => {
//                 "ots_getBlockTransactions".hash(state);
//                 block_number_or_tag.hash(state);
//                 page.hash(state);
//                 page_size.hash(state);
//             }
//             EthRequest::OtsSearchTransactionsBefore(address, block_number_or_tag, page) => {
//                 "ots_searchTransactionsBefore".hash(state);
//                 address.hash(state);
//                 block_number_or_tag.hash(state);
//                 page.hash(state);
//             }
//             EthRequest::OtsSearchTransactionsAfter(address, block_number_or_tag, page) => {
//                 "ots_searchTransactionsAfter".hash(state);
//                 address.hash(state);
//                 block_number_or_tag.hash(state);
//                 page.hash(state);
//             }
//             EthRequest::OtsGetTransactionBySenderAndNonce(address, uint) => {
//                 "ots_getTransactionBySenderAndNonce".hash(state);
//                 address.hash(state);
//                 uint.hash(state);
//             }
//             EthRequest::OtsGetContractCreator(address) => {
//                 "ots_getContractCreator".hash(state);
//                 address.hash(state);
//             }

//             // ***** EXOTIC *****
//             EthRequest::EvmMine(_) => {
//                 // no need to hash
//             }

//             // ***** ANVIL *****
//             EthRequest::ImpersonateAccount(_)
//             | EthRequest::StopImpersonatingAccount(_)
//             | EthRequest::AutoImpersonateAccount(_)
//             | EthRequest::GetAutoMine(_)
//             | EthRequest::Mine(_, _)
//             | EthRequest::SetAutomine(_)
//             | EthRequest::SetIntervalMining(_)
//             | EthRequest::GetIntervalMining(_)
//             | EthRequest::DropTransaction(_)
//             | EthRequest::DropAllTransactions(_)
//             | EthRequest::Reset(_)
//             | EthRequest::SetRpcUrl(_)
//             | EthRequest::SetBalance(_, _)
//             | EthRequest::SetCode(_, _)
//             | EthRequest::SetNonce(_, _)
//             | EthRequest::SetStorageAt(_, _, _)
//             | EthRequest::SetCoinbase(_)
//             | EthRequest::SetChainId(_)
//             | EthRequest::SetLogging(_)
//             | EthRequest::SetMinGasPrice(_)
//             | EthRequest::SetNextBlockBaseFeePerGas(_)
//             | EthRequest::DumpState(_)
//             | EthRequest::LoadState(_)
//             | EthRequest::NodeInfo(_)
//             | EthRequest::AnvilMetadata(_)
//             | EthRequest::EvmSnapshot(_)
//             | EthRequest::EvmRevert(_)
//             | EthRequest::EvmIncreaseTime(_)
//             | EthRequest::EvmSetNextBlockTimeStamp(_)
//             | EthRequest::EvmSetBlockGasLimit(_)
//             | EthRequest::EvmSetBlockTimeStampInterval(_)
//             | EthRequest::EvmRemoveBlockTimeStampInterval(_)
//             | EthRequest::EvmMineDetailed(_)
//             | EthRequest::EnableTraces(_)
//             | EthRequest::RemovePoolTransactions(_)
//             | EthRequest::Reorg(_)
//             | EthRequest::Rollback(_)
//             | EthRequest::AnvilAddCapability(_)
//             | EthRequest::EvmSetTime(_)
//             | EthRequest::AnvilSetExecutor(_) => {
//                 // these are anvil specific requests, no need to hash
//             }

//             // ***** Wallet *****
//             EthRequest::WalletGetCapabilities(_) | EthRequest::WalletSendTransaction(_) => {
//                 // no need to hash
//             }
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_web3_client_version() {
//         let s = r#"{"method": "web3_clientVersion", "params":[]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_web3_sha3() {
//         let s = r#"{"method": "web3_sha3", "params":["0x68656c6c6f20776f726c64"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_eth_accounts() {
//         let s = r#"{"method": "eth_accounts", "params":[]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_eth_network_id() {
//         let s = r#"{"method": "eth_networkId", "params":[]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_eth_get_proof() {
//         let s = r#"{"method":"eth_getProof","params":["0x7F0d15C7FAae65896648C8273B6d7E43f58Fa842",["0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"],"latest"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_eth_chain_id() {
//         let s = r#"{"method": "eth_chainId", "params":[]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_net_listening() {
//         let s = r#"{"method": "net_listening", "params":[]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_eth_block_number() {
//         let s = r#"{"method": "eth_blockNumber", "params":[]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_eth_max_priority_fee() {
//         let s = r#"{"method": "eth_maxPriorityFeePerGas", "params":[]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_eth_syncing() {
//         let s = r#"{"method": "eth_syncing", "params":[]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_custom_impersonate_account() {
//         let s = r#"{"method": "anvil_impersonateAccount", "params":
// ["0xd84de507f3fada7df80908082d3239466db55a71"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_custom_stop_impersonate_account() {
//         let s = r#"{"method": "anvil_stopImpersonatingAccount",  "params":
// ["0x364d6D0333432C3Ac016Ca832fb8594A8cE43Ca6"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_custom_auto_impersonate_account() {
//         let s = r#"{"method": "anvil_autoImpersonateAccount",  "params": [true]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_custom_get_automine() {
//         let s = r#"{"method": "anvil_getAutomine", "params": []}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_custom_mine() {
//         let s = r#"{"method": "anvil_mine", "params": []}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::Mine(num, time) => {
//                 assert!(num.is_none());
//                 assert!(time.is_none());
//             }
//             _ => unreachable!(),
//         }
//         let s = r#"{"method": "anvil_mine", "params":
// ["0xd84de507f3fada7df80908082d3239466db55a71"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::Mine(num, time) => {
//                 assert!(num.is_some());
//                 assert!(time.is_none());
//             }
//             _ => unreachable!(),
//         }
//         let s = r#"{"method": "anvil_mine", "params": ["0xd84de507f3fada7df80908082d3239466db55a71", "0xd84de507f3fada7df80908082d3239466db55a71"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::Mine(num, time) => {
//                 assert!(num.is_some());
//                 assert!(time.is_some());
//             }
//             _ => unreachable!(),
//         }
//     }

//     #[test]
//     fn test_custom_auto_mine() {
//         let s = r#"{"method": "anvil_setAutomine", "params": [false]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//         let s = r#"{"method": "evm_setAutomine", "params": [false]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_custom_interval_mining() {
//         let s = r#"{"method": "anvil_setIntervalMining", "params": [100]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//         let s = r#"{"method": "evm_setIntervalMining", "params": [100]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_custom_drop_tx() {
//         let s = r#"{"method": "anvil_dropTransaction", "params":
// ["0x4a3b0fce2cb9707b0baa68640cf2fe858c8bb4121b2a8cb904ff369d38a560ff"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_custom_reset() {
//         let s = r#"{"method": "anvil_reset", "params": [{"forking": {"jsonRpcUrl": "https://ethereumpublicnode.com",
//         "blockNumber": "18441649"
//       }
//     }]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::Reset(forking) => {
//                 let forking = forking.and_then(|f| f.params);
//                 assert_eq!(
//                     forking,
//                     Some(Forking {
//                         json_rpc_url: Some("https://ethereumpublicnode.com".into()),
//                         block_number: Some(18441649)
//                     })
//                 )
//             }
//             _ => unreachable!(),
//         }

//         let s = r#"{"method": "anvil_reset", "params": [ { "forking": {
//                 "jsonRpcUrl": "https://eth-mainnet.alchemyapi.io/v2/<key>",
//                 "blockNumber": 11095000
//         }}]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::Reset(forking) => {
//                 let forking = forking.and_then(|f| f.params);
//                 assert_eq!(
//                     forking,
//                     Some(Forking {
//                         json_rpc_url: Some(
//                             "https://eth-mainnet.alchemyapi.io/v2/<key>".to_string()
//                         ),
//                         block_number: Some(11095000)
//                     })
//                 )
//             }
//             _ => unreachable!(),
//         }

//         let s = r#"{"method": "anvil_reset", "params": [ { "forking": {
//                 "jsonRpcUrl": "https://eth-mainnet.alchemyapi.io/v2/<key>"
//         }}]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::Reset(forking) => {
//                 let forking = forking.and_then(|f| f.params);
//                 assert_eq!(
//                     forking,
//                     Some(Forking {
//                         json_rpc_url: Some(
//                             "https://eth-mainnet.alchemyapi.io/v2/<key>".to_string()
//                         ),
//                         block_number: None
//                     })
//                 )
//             }
//             _ => unreachable!(),
//         }

//         let s = r#"{"method":"anvil_reset","params":[{"jsonRpcUrl": "http://localhost:8545", "blockNumber": 14000000}]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::Reset(forking) => {
//                 let forking = forking.and_then(|f| f.params);
//                 assert_eq!(
//                     forking,
//                     Some(Forking {
//                         json_rpc_url: Some("http://localhost:8545".to_string()),
//                         block_number: Some(14000000)
//                     })
//                 )
//             }
//             _ => unreachable!(),
//         }

//         let s = r#"{"method":"anvil_reset","params":[{ "blockNumber": 14000000}]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::Reset(forking) => {
//                 let forking = forking.and_then(|f| f.params);
//                 assert_eq!(
//                     forking,
//                     Some(Forking {
//                         json_rpc_url: None,
//                         block_number: Some(14000000)
//                     })
//                 )
//             }
//             _ => unreachable!(),
//         }

//         let s = r#"{"method":"anvil_reset","params":[{ "blockNumber": "14000000"}]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::Reset(forking) => {
//                 let forking = forking.and_then(|f| f.params);
//                 assert_eq!(
//                     forking,
//                     Some(Forking {
//                         json_rpc_url: None,
//                         block_number: Some(14000000)
//                     })
//                 )
//             }
//             _ => unreachable!(),
//         }

//         let s = r#"{"method":"anvil_reset","params":[{"jsonRpcUrl": "http://localhost:8545"}]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::Reset(forking) => {
//                 let forking = forking.and_then(|f| f.params);
//                 assert_eq!(
//                     forking,
//                     Some(Forking {
//                         json_rpc_url: Some("http://localhost:8545".to_string()),
//                         block_number: None
//                     })
//                 )
//             }
//             _ => unreachable!(),
//         }

//         let s = r#"{"method": "anvil_reset"}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::Reset(forking) => {
//                 assert!(forking.is_none())
//             }
//             _ => unreachable!(),
//         }
//     }

//     #[test]
//     fn test_custom_set_balance() {
//         let s = r#"{"method": "anvil_setBalance", "params":
// ["0xd84de507f3fada7df80908082d3239466db55a71", "0x0"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "anvil_setBalance", "params":
// ["0xd84de507f3fada7df80908082d3239466db55a71", 1337]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_custom_set_code() {
//         let s = r#"{"method": "anvil_setCode", "params":
// ["0xd84de507f3fada7df80908082d3239466db55a71", "0x0123456789abcdef"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "anvil_setCode", "params":
// ["0xd84de507f3fada7df80908082d3239466db55a71", "0x"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "anvil_setCode", "params":
// ["0xd84de507f3fada7df80908082d3239466db55a71", ""]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_custom_set_nonce() {
//         let s = r#"{"method": "anvil_setNonce", "params":
// ["0xd84de507f3fada7df80908082d3239466db55a71", "0x0"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//         let s = r#"{"method":
// "hardhat_setNonce", "params": ["0xd84de507f3fada7df80908082d3239466db55a71", "0x0"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//         let s = r#"{"method": "evm_setAccountNonce", "params":
// ["0xd84de507f3fada7df80908082d3239466db55a71", "0x0"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_custom_set_storage_at() {
//         let s = r#"{"method": "anvil_setStorageAt", "params":
// ["0x295a70b2de5e3953354a6a8344e616ed314d7251", "0x0",
// "0x0000000000000000000000000000000000000000000000000000000000003039"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "hardhat_setStorageAt", "params":
// ["0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56",
// "0xa6eef7e35abe7026729641147f7915573c7e97b47efa546f5f6e3230263bcb49",
// "0x0000000000000000000000000000000000000000000000000000000000003039"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_custom_coinbase() {
//         let s = r#"{"method": "anvil_setCoinbase", "params":
// ["0x295a70b2de5e3953354a6a8344e616ed314d7251"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_custom_logging() {
//         let s = r#"{"method": "anvil_setLoggingEnabled", "params": [false]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_custom_min_gas_price() {
//         let s = r#"{"method": "anvil_setMinGasPrice", "params": ["0x0"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_custom_next_block_base_fee() {
//         let s = r#"{"method": "anvil_setNextBlockBaseFeePerGas", "params": ["0x0"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_set_time() {
//         let s = r#"{"method": "anvil_setTime", "params": ["0x0"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "anvil_increaseTime", "params": 1}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_custom_dump_state() {
//         let s = r#"{"method": "anvil_dumpState", "params": [true]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "anvil_dumpState"}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::DumpState(param) => {
//                 assert!(param.is_none());
//             }
//             _ => unreachable!(),
//         }
//     }

//     #[test]
//     fn test_serde_custom_load_state() {
//         let s = r#"{"method": "anvil_loadState", "params": ["0x0001"] }"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_custom_snapshot() {
//         let s = r#"{"method": "anvil_snapshot", "params": [] }"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "evm_snapshot", "params": [] }"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_custom_revert() {
//         let s = r#"{"method": "anvil_revert", "params": ["0x0"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_custom_increase_time() {
//         let s = r#"{"method": "anvil_increaseTime", "params": ["0x0"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "anvil_increaseTime", "params": [1]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "anvil_increaseTime", "params": 1}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "evm_increaseTime", "params": ["0x0"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "evm_increaseTime", "params": [1]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "evm_increaseTime", "params": 1}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_custom_next_timestamp() {
//         let s = r#"{"method": "anvil_setNextBlockTimestamp", "params": [100]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//         let s = r#"{"method": "evm_setNextBlockTimestamp", "params": [100]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//         let s = r#"{"method": "evm_setNextBlockTimestamp", "params": ["0x64e0f308"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_custom_timestamp_interval() {
//         let s = r#"{"method": "anvil_setBlockTimestampInterval", "params": [100]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_custom_remove_timestamp_interval() {
//         let s = r#"{"method": "anvil_removeBlockTimestampInterval", "params": []}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_custom_evm_mine() {
//         let s = r#"{"method": "evm_mine", "params": [100]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//         let s = r#"{"method": "evm_mine", "params": [{
//             "timestamp": 100,
//             "blocks": 100
//         }]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::EvmMine(params) => {
//                 assert_eq!(
//                     params.unwrap().params.unwrap_or_default(),
//                     MineOptions::Options {
//                         timestamp: Some(100),
//                         blocks: Some(100)
//                     }
//                 )
//             }
//             _ => unreachable!(),
//         }

//         let s = r#"{"method": "evm_mine"}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();

//         match req {
//             EthRequest::EvmMine(params) => {
//                 assert!(params.is_none())
//             }
//             _ => unreachable!(),
//         }

//         let s = r#"{"method": "evm_mine", "params": []}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_custom_evm_mine_detailed() {
//         let s = r#"{"method": "anvil_mine_detailed", "params": [100]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//         let s = r#"{"method": "anvil_mine_detailed", "params": [{
//             "timestamp": 100,
//             "blocks": 100
//         }]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::EvmMineDetailed(params) => {
//                 assert_eq!(
//                     params.unwrap().params.unwrap_or_default(),
//                     MineOptions::Options {
//                         timestamp: Some(100),
//                         blocks: Some(100)
//                     }
//                 )
//             }
//             _ => unreachable!(),
//         }

//         let s = r#"{"method": "evm_mine_detailed"}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();

//         match req {
//             EthRequest::EvmMineDetailed(params) => {
//                 assert!(params.is_none())
//             }
//             _ => unreachable!(),
//         }

//         let s = r#"{"method": "anvil_mine_detailed", "params": []}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_custom_evm_mine_hex() {
//         let s = r#"{"method": "evm_mine", "params": ["0x63b6ff08"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::EvmMine(params) => {
//                 assert_eq!(
//                     params.unwrap().params.unwrap_or_default(),
//                     MineOptions::Timestamp(Some(1672937224))
//                 )
//             }
//             _ => unreachable!(),
//         }

//         let s = r#"{"method": "evm_mine", "params": [{"timestamp": "0x63b6ff08"}]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let req = serde_json::from_value::<EthRequest>(value).unwrap();
//         match req {
//             EthRequest::EvmMine(params) => {
//                 assert_eq!(
//                     params.unwrap().params.unwrap_or_default(),
//                     MineOptions::Options {
//                         timestamp: Some(1672937224),
//                         blocks: None
//                     }
//                 )
//             }
//             _ => unreachable!(),
//         }
//     }

//     #[test]
//     fn test_eth_uncle_count_by_block_hash() {
//         let s = r#"{"jsonrpc":"2.0","method":"eth_getUncleCountByBlockHash","params":["0x4a3b0fce2cb9707b0baa68640cf2fe858c8bb4121b2a8cb904ff369d38a560ff"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_eth_block_tx_count_by_block_hash() {
//         let s = r#"{"jsonrpc":"2.0","method":"eth_getBlockTransactionCountByHash","params":["0x4a3b0fce2cb9707b0baa68640cf2fe858c8bb4121b2a8cb904ff369d38a560ff"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_eth_get_logs() {
//         let s = r#"{"jsonrpc":"2.0","method":"eth_getLogs","params":[{"topics":["0x000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b"]}],"id":74}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_eth_new_filter() {
//         let s = r#"{"method": "eth_newFilter", "params": [{"topics":["0x000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b"]}],"id":73}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_eth_unsubscribe() {
//         let s = r#"{"id": 1, "method": "eth_unsubscribe", "params":
// ["0x9cef478923ff08bf67fde6c64013158d"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthPubSub>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_eth_subscribe() {
//         let s = r#"{"id": 1, "method": "eth_subscribe", "params": ["newHeads"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthPubSub>(value).unwrap();

//         let s = r#"{"id": 1, "method": "eth_subscribe", "params": ["logs", {"address":
// "0x8320fe7702b96808f7bbc0d4a888ed1468216cfd", "topics":
// ["0xd78a0cb8bb633d06981248b816e7bd33c2a35a6089241d099fa519e361cab902"]}]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthPubSub>(value).unwrap();

//         let s = r#"{"id": 1, "method": "eth_subscribe", "params": ["newPendingTransactions"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthPubSub>(value).unwrap();

//         let s = r#"{"id": 1, "method": "eth_subscribe", "params": ["syncing"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthPubSub>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_debug_raw_transaction() {
//         let s = r#"{"jsonrpc":"2.0","method":"debug_getRawTransaction","params":["0x3ed3a89bc10115a321aee238c02de214009f8532a65368e5df5eaf732ee7167c"],"id":1}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"jsonrpc":"2.0","method":"eth_getRawTransactionByHash","params":["0x3ed3a89bc10115a321aee238c02de214009f8532a65368e5df5eaf732ee7167c"],"id":1}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"jsonrpc":"2.0","method":"eth_getRawTransactionByBlockHashAndIndex","params":["0x3ed3a89bc10115a321aee238c02de214009f8532a65368e5df5eaf732ee7167c",1],"id":1}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"jsonrpc":"2.0","method":"eth_getRawTransactionByBlockNumberAndIndex","params":["0x3ed3a89b",0],"id":1}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_debug_trace_transaction() {
//         let s = r#"{"method": "debug_traceTransaction", "params":
// ["0x4a3b0fce2cb9707b0baa68640cf2fe858c8bb4121b2a8cb904ff369d38a560ff"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "debug_traceTransaction", "params":
// ["0x4a3b0fce2cb9707b0baa68640cf2fe858c8bb4121b2a8cb904ff369d38a560ff", {}]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "debug_traceTransaction", "params":
// ["0x4a3b0fce2cb9707b0baa68640cf2fe858c8bb4121b2a8cb904ff369d38a560ff", {"disableStorage":
// true}]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_debug_trace_call() {
//         let s = r#"{"method": "debug_traceCall", "params": [{"data":"0xcfae3217","from":"0xd84de507f3fada7df80908082d3239466db55a71","to":"0xcbe828fdc46e3b1c351ec90b1a5e7d9742c0398d"}]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "debug_traceCall", "params": [{"data":"0xcfae3217","from":"0xd84de507f3fada7df80908082d3239466db55a71","to":"0xcbe828fdc46e3b1c351ec90b1a5e7d9742c0398d"}, { "blockNumber": "latest" }]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "debug_traceCall", "params": [{"data":"0xcfae3217","from":"0xd84de507f3fada7df80908082d3239466db55a71","to":"0xcbe828fdc46e3b1c351ec90b1a5e7d9742c0398d"}, { "blockNumber": "0x0" }]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "debug_traceCall", "params": [{"data":"0xcfae3217","from":"0xd84de507f3fada7df80908082d3239466db55a71","to":"0xcbe828fdc46e3b1c351ec90b1a5e7d9742c0398d"}, { "blockHash": "0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3" }]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         let s = r#"{"method": "debug_traceCall", "params": [{"data":"0xcfae3217","from":"0xd84de507f3fada7df80908082d3239466db55a71","to":"0xcbe828fdc46e3b1c351ec90b1a5e7d9742c0398d"}, { "blockNumber": "0x0" }, {"disableStorage": true}]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_eth_storage() {
//         let s = r#"{"method": "eth_getStorageAt", "params":
// ["0x295a70b2de5e3953354a6a8344e616ed314d7251", "0x0", "latest"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_eth_call() {
//         let req = r#"{"data":"0xcfae3217","from":"0xd84de507f3fada7df80908082d3239466db55a71","to":"0xcbe828fdc46e3b1c351ec90b1a5e7d9742c0398d"}"#;
//         let _req = serde_json::from_str::<TransactionRequest>(req).unwrap();

//         let s = r#"{"method": "eth_call", "params":[{"data":"0xcfae3217","from":"0xd84de507f3fada7df80908082d3239466db55a71","to":"0xcbe828fdc46e3b1c351ec90b1a5e7d9742c0398d"},"latest"]}"#;
//         let _req = serde_json::from_str::<EthRequest>(s).unwrap();

//         let s = r#"{"method": "eth_call", "params":[{"data":"0xcfae3217","from":"0xd84de507f3fada7df80908082d3239466db55a71","to":"0xcbe828fdc46e3b1c351ec90b1a5e7d9742c0398d"}]}"#;
//         let _req = serde_json::from_str::<EthRequest>(s).unwrap();

//         let s = r#"{"method": "eth_call", "params":[{"data":"0xcfae3217","from":"0xd84de507f3fada7df80908082d3239466db55a71","to":"0xcbe828fdc46e3b1c351ec90b1a5e7d9742c0398d"}, { "blockNumber": "latest" }]}"#;
//         let _req = serde_json::from_str::<EthRequest>(s).unwrap();

//         let s = r#"{"method": "eth_call", "params":[{"data":"0xcfae3217","from":"0xd84de507f3fada7df80908082d3239466db55a71","to":"0xcbe828fdc46e3b1c351ec90b1a5e7d9742c0398d"}, { "blockNumber": "0x0" }]}"#;
//         let _req = serde_json::from_str::<EthRequest>(s).unwrap();

//         let s = r#"{"method": "eth_call", "params":[{"data":"0xcfae3217","from":"0xd84de507f3fada7df80908082d3239466db55a71","to":"0xcbe828fdc46e3b1c351ec90b1a5e7d9742c0398d"}, { "blockHash":"0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3" }]}"#;
//         let _req = serde_json::from_str::<EthRequest>(s).unwrap();
//     }

//     #[test]
//     fn test_serde_eth_balance() {
//         let s = r#"{"method": "eth_getBalance", "params":
// ["0x295a70b2de5e3953354a6a8344e616ed314d7251", "latest"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();

//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_eth_block_by_number() {
//         let s = r#"{"method": "eth_getBlockByNumber", "params": ["0x0", true]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//         let s = r#"{"method": "eth_getBlockByNumber", "params": ["latest", true]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//         let s = r#"{"method": "eth_getBlockByNumber", "params": ["earliest", true]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//         let s = r#"{"method": "eth_getBlockByNumber", "params": ["pending", true]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();

//         // this case deviates from the spec, but we're supporting this for legacy reasons: <https://github.com/foundry-rs/foundry/issues/1868>
//         let s = r#"{"method": "eth_getBlockByNumber", "params": [0, true]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_eth_sign() {
//         let s = r#"{"method": "eth_sign", "params":
// ["0xd84de507f3fada7df80908082d3239466db55a71", "0x00"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//         let s = r#"{"method": "personal_sign", "params":
// ["0x00", "0xd84de507f3fada7df80908082d3239466db55a71"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_eth_sign_typed_data() {
//         let s = r#"{"method":"eth_signTypedData_v4","params":["0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826", {"types":{"EIP712Domain":[{"name":"name","type":"string"},{"name":"version","type":"string"},{"name":"chainId","type":"uint256"},{"name":"verifyingContract","type":"address"}],"Person":[{"name":"name","type":"string"},{"name":"wallet","type":"address"}],"Mail":[{"name":"from","type":"Person"},{"name":"to","type":"Person"},{"name":"contents","type":"string"}]},"primaryType":"Mail","domain":{"name":"Ether Mail","version":"1","chainId":1,"verifyingContract":"0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"},"message":{"from":{"name":"Cow","wallet":"0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"},"to":{"name":"Bob","wallet":"0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"},"contents":"Hello, Bob!"}}]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_remove_pool_transactions() {
//         let s = r#"{"method": "anvil_removePoolTransactions",  "params":["0x364d6D0333432C3Ac016Ca832fb8594A8cE43Ca6"]}"#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }

//     #[test]
//     fn test_serde_anvil_reorg() {
//         // TransactionData::JSON
//         let s = r#"
//         {
//             "method": "anvil_reorg",
//             "params": [
//                 5,
//                 [
//                     [
//                         {
//                             "from": "0x976EA74026E726554dB657fA54763abd0C3a0aa9",
//                             "to": "0x1199bc69f16FDD6690DC40339EC445FaE1b6DD11",
//                             "value": 100
//                         },
//                         1
//                     ],
//                     [
//                         {
//                             "from": "0x976EA74026E726554dB657fA54763abd0C3a0aa9",
//                             "to": "0x1199bc69f16FDD6690DC40339EC445FaE1b6DD11",
//                             "value": 200
//                         },
//                         2
//                     ]
//                 ]
//             ]
//         }
//         "#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//         // TransactionData::Raw
//         let s = r#"
//         {
//             "method": "anvil_reorg",
//             "params": [
//                 5,
//                 [
//                     [
//                         "0x19d55c67e1ba8f1bbdfed75f8ad524ebf087e4ecb848a2d19881d7a5e3d2c54e1732cb1b462da3b3fdb05bdf4c4d3c8e3c9fcebdc2ab5fa5d59a3f752888f27e1b",
//                         1
//                     ]
//                 ]
//             ]
//         }
//         "#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//         // TransactionData::Raw and TransactionData::JSON
//         let s = r#"
//         {
//             "method": "anvil_reorg",
//             "params": [
//                 5,
//                 [
//                     [
//                         "0x19d55c67e1ba8f1bbdfed75f8ad524ebf087e4ecb848a2d19881d7a5e3d2c54e1732cb1b462da3b3fdb05bdf4c4d3c8e3c9fcebdc2ab5fa5d59a3f752888f27e1b",
//                         1
//                     ],
//                     [
//                         {
//                             "from": "0x976EA74026E726554dB657fA54763abd0C3a0aa9",
//                             "to": "0x1199bc69f16FDD6690DC40339EC445FaE1b6DD11",
//                             "value": 200
//                         },
//                         2
//                     ]
//                 ]
//             ]
//         }
//         "#;
//         let value: serde_json::Value = serde_json::from_str(s).unwrap();
//         let _req = serde_json::from_value::<EthRequest>(value).unwrap();
//     }
// }
