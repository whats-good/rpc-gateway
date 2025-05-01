use std::{sync::Arc, time::Duration};

use alloy_eips::{BlockId, BlockNumberOrTag};
use arc_swap::ArcSwap;
use rpc_gateway_eth::eth::EthRequest;

static ONE_YEAR: Duration = Duration::from_secs(31536000);

/// Manages the TTL for the cache
#[derive(Debug)]
pub struct TTLManager {
    block_time: Duration,
    /// The latest block number for this chain
    latest_block_number: ArcSwap<u64>,
}

impl TTLManager {
    pub fn new(block_time: Duration) -> Self {
        Self {
            block_time,
            latest_block_number: ArcSwap::new(Arc::new(0)),
        }
    }

    pub fn get_ttl_from_block_number_or_tag(
        &self,
        block_number_or_tag: &BlockNumberOrTag,
    ) -> Option<Duration> {
        let block_time = self.block_time;
        match block_number_or_tag {
            BlockNumberOrTag::Latest => Some(block_time.clone()),
            BlockNumberOrTag::Finalized => Some(ONE_YEAR),
            BlockNumberOrTag::Safe => Some(block_time.clone()), // TODO: can do better here
            BlockNumberOrTag::Earliest => Some(ONE_YEAR),
            BlockNumberOrTag::Pending => None,
            BlockNumberOrTag::Number(number) => {
                let latest_block_number = self.get_latest_block_number();
                if *number < latest_block_number && latest_block_number - number > 50 {
                    // TODO: can cache block with longer diff a bit longer than the rest. revisit this part.
                    Some(ONE_YEAR)
                } else {
                    Some(block_time.clone())
                }
            }
        }
    }

    pub fn get_ttl_from_block_id(&self, block_id: &BlockId) -> Option<Duration> {
        let block_number_or_tag = match block_id {
            BlockId::Hash(_) => return Some(ONE_YEAR), // can cache hash for a very long time - 1 year in seconds
            BlockId::Number(block_number_or_tag) => block_number_or_tag,
        };
        self.get_ttl_from_block_number_or_tag(block_number_or_tag)
    }

    pub fn get_ttl(&self, req: &EthRequest) -> Option<Duration> {
        let block_time = self.block_time;
        match req {
            EthRequest::EthNetworkId(_) => Some(ONE_YEAR),
            EthRequest::EthGasPrice(_) => Some(block_time.clone()), // TODO: make this configurable
            EthRequest::EthMaxPriorityFeePerGas(_) => Some(block_time.clone()), // TODO: make this configurable
            EthRequest::EthBlobBaseFee(_) => Some(block_time.clone()), // TODO: make this configurable
            EthRequest::EthBlockNumber(_) => Some(block_time.clone()), // TODO: make this configurable
            EthRequest::EthGetBalance(_, block_id) => block_id
                .and_then(|block_id| self.get_ttl_from_block_id(&block_id))
                .or(Some(block_time.clone())),
            EthRequest::EthGetAccount(_, block_id) => block_id
                .and_then(|block_id| self.get_ttl_from_block_id(&block_id))
                .or(Some(block_time.clone())),
            EthRequest::EthGetStorageAt(_, _, block_id) => block_id
                .and_then(|block_id| self.get_ttl_from_block_id(&block_id))
                .or(Some(block_time.clone())),
            EthRequest::EthGetBlockByHash(_, _) => Some(ONE_YEAR),
            EthRequest::EthGetBlockByNumber(block_number_or_tag, _) => {
                self.get_ttl_from_block_number_or_tag(block_number_or_tag)
            }
            EthRequest::EthGetTransactionCount(_, block_id) => block_id
                .and_then(|block_id| self.get_ttl_from_block_id(&block_id))
                .or(Some(block_time.clone())),
            EthRequest::EthGetTransactionCountByHash(_) => Some(ONE_YEAR),
            EthRequest::EthGetTransactionCountByNumber(block_number_or_tag) => {
                self.get_ttl_from_block_number_or_tag(block_number_or_tag)
            }
            EthRequest::EthGetUnclesCountByHash(_) => Some(ONE_YEAR),
            EthRequest::EthGetUnclesCountByNumber(block_number_or_tag) => {
                self.get_ttl_from_block_number_or_tag(block_number_or_tag)
            }
            EthRequest::EthGetCodeAt(_, block_id) => block_id
                .and_then(|block_id| self.get_ttl_from_block_id(&block_id))
                .or(Some(block_time.clone())),
            EthRequest::EthGetProof(_, _, block_id) => block_id
                .and_then(|block_id| self.get_ttl_from_block_id(&block_id))
                .or(Some(block_time.clone())),
            EthRequest::EthCall(_, block_id, _) => block_id
                .and_then(|block_id| self.get_ttl_from_block_id(&block_id))
                .or(Some(block_time.clone())),
            EthRequest::EthEstimateGas(_, block_id, _) => block_id
                .and_then(|block_id| self.get_ttl_from_block_id(&block_id))
                .or(Some(block_time.clone())),
            EthRequest::EthGetTransactionByHash(_) => Some(ONE_YEAR),
            EthRequest::EthGetTransactionByBlockHashAndIndex(_, _) => Some(ONE_YEAR),
            EthRequest::EthGetTransactionByBlockNumberAndIndex(block_number_or_tag, _) => {
                self.get_ttl_from_block_number_or_tag(block_number_or_tag)
            }
            EthRequest::EthGetRawTransactionByHash(_) => Some(ONE_YEAR),
            EthRequest::EthGetRawTransactionByBlockHashAndIndex(_, _) => Some(ONE_YEAR),
            EthRequest::EthGetRawTransactionByBlockNumberAndIndex(block_number_or_tag, _) => {
                self.get_ttl_from_block_number_or_tag(block_number_or_tag)
            }
            EthRequest::EthGetTransactionReceipt(_) => {
                // TODO: this actually depends on the transaction itself. sometimes the ttl needs to be aware of the data we're writing.
                // We're currently re-using ttls to determine if we should cache the response - or whether the request could even exist in the cache in the first place. So we need to separate the two, and actually take a look at the data we're writing while determining the final ttl.
                Some(block_time.clone())
            }
            EthRequest::EthGetBlockReceipts(block_id) => self.get_ttl_from_block_id(block_id),
            EthRequest::EthGetUncleByBlockHashAndIndex(_, _) => Some(ONE_YEAR),
            EthRequest::EthGetUncleByBlockNumberAndIndex(block_number_or_tag, _) => {
                self.get_ttl_from_block_number_or_tag(block_number_or_tag)
            }
            EthRequest::EthGetLogs(_) => Some(block_time.clone()),
            EthRequest::EthGetFilterChanges(_) => Some(block_time.clone()),
            EthRequest::EthGetFilterLogs(_) => Some(block_time.clone()),
            EthRequest::EthFeeHistory(_, block_number_or_tag, _) => {
                self.get_ttl_from_block_number_or_tag(block_number_or_tag)
            }
            _ => None,
        }
    }

    fn get_latest_block_number(&self) -> u64 {
        **self.latest_block_number.load()
    }
}
