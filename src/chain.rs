use serde::Deserialize;
use std::hash::Hash;
use std::time::Duration;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Deserialize)]
pub struct ChainId(u64);

impl From<u64> for ChainId {
    fn from(chain_id: u64) -> Self {
        ChainId(chain_id)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Chain {
    pub block_time: Option<Duration>,
    pub chain_id: ChainId,
}

impl PartialEq for Chain {
    fn eq(&self, other: &Self) -> bool {
        self.chain_id == other.chain_id
    }
}

impl Eq for Chain {}

impl Hash for Chain {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.chain_id.hash(state);
    }
}

impl From<ChainId> for Chain {
    fn from(value: ChainId) -> Self {
        Chain::new(value, None)
    }
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
