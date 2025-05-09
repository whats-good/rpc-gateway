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
            EthRequest::EthNetworkId { .. } => Some(ONE_YEAR),
            EthRequest::EthGasPrice { .. } => Some(block_time.clone()), // TODO: make this configurable
            EthRequest::EthMaxPriorityFeePerGas { .. } => Some(block_time.clone()), // TODO: make this configurable
            EthRequest::EthBlobBaseFee { .. } => Some(block_time.clone()), // TODO: make this configurable
            EthRequest::EthBlockNumber { .. } => Some(block_time.clone()), // TODO: make this configurable
            EthRequest::EthGetBalance { params: p } => p
                .block_id
                .and_then(|block_id| self.get_ttl_from_block_id(&block_id))
                .or(Some(block_time.clone())),
            EthRequest::EthGetStorageAt { params: p } => p
                .block_id
                .and_then(|block_id| self.get_ttl_from_block_id(&block_id))
                .or(Some(block_time.clone())),
            EthRequest::EthGetBlockByHash { .. } => Some(ONE_YEAR),
            EthRequest::EthGetBlockByNumber { params: p } => {
                self.get_ttl_from_block_number_or_tag(&p.block_number)
            }
            EthRequest::EthGetTransactionCount { params: p } => p
                .block_id
                .and_then(|block_id| self.get_ttl_from_block_id(&block_id))
                .or(Some(block_time.clone())),
            EthRequest::EthGetCodeAt { params: p } => p
                .block_id
                .and_then(|block_id| self.get_ttl_from_block_id(&block_id))
                .or(Some(block_time.clone())),
            EthRequest::EthCall { params: p } => p
                .block_id
                .and_then(|block_id| self.get_ttl_from_block_id(&block_id))
                .or(Some(block_time.clone())),
            EthRequest::EthEstimateGas { params: p } => p
                .block_id
                .and_then(|block_id| self.get_ttl_from_block_id(&block_id))
                .or(Some(block_time.clone())),
            EthRequest::EthGetTransactionReceipt { params: p } => Some(block_time.clone()),
            EthRequest::EthGetLogs { .. } => Some(block_time.clone()), // TODO: this should be based on the filter.

            // These are canned, so we exclude them here.
            EthRequest::EthChainId { .. } => None,
            EthRequest::Web3ClientVersion { .. } => None,
        }
    }

    fn get_latest_block_number(&self) -> u64 {
        **self.latest_block_number.load()
    }
}
