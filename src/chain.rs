use std::time::Duration;

use derive_more::derive::Constructor;
use serde::Deserialize;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Deserialize)]
pub struct ChainId(u64);

impl From<u64> for ChainId {
    fn from(chain_id: u64) -> Self {
        ChainId(chain_id)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Constructor, Debug, Deserialize)]
pub struct Chain {
    pub block_time: Option<Duration>,
    pub chain_id: ChainId,
}

impl Chain {
    pub fn leak(self) -> &'static Self {
        Box::leak(Box::new(self))
    }
}
