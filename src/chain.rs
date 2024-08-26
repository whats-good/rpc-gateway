use std::time::Duration;

use derive_more::derive::Constructor;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct ChainId(u64);

impl From<u64> for ChainId {
    fn from(chain_id: u64) -> Self {
        ChainId(chain_id)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Constructor, Debug)]
pub struct Chain {
    name: String,
    block_time: Duration,
    chain_id: ChainId,
}

impl Chain {
    pub fn leak(self) -> &'static Self {
        Box::leak(Box::new(self))
    }
}
