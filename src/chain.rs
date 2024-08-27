use serde::Deserialize;
use std::time::Duration;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Deserialize)]
pub struct ChainId(u64);

impl From<u64> for ChainId {
    fn from(chain_id: u64) -> Self {
        ChainId(chain_id)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Deserialize)]
pub struct Chain {
    pub block_time: Option<Duration>,
    pub chain_id: ChainId,
}

impl Chain {
    pub fn new(chain_id: ChainId, block_time: Option<Duration>) -> Self {
        Self {
            chain_id,
            block_time,
        }
    }

    pub fn leak(self) -> &'static Self {
        Box::leak(Box::new(self))
    }
}
