use anvil_core::eth::EthRequest;
use rpc_gateway_config::{CacheConfig, ChainConfig};
use std::{sync::Arc, time::Duration};
use tracing::{error, warn};

use crate::{local_cache::LocalCache, redis::RedisCache, ttl::TTLManager};

// TODO: this should not be async
pub async fn from_config(
    cache_config: &CacheConfig,
    chain_config: &ChainConfig,
) -> Option<RpcCache> {
    let block_time = match chain_config.block_time {
        Some(block_time) => block_time,
        None => {
            error!(
                chain = ?chain_config.chain,
                "Cache enabled but no block time available. Disabling cache."
            );
            return None;
        }
    };
    let ttl_manager = TTLManager::new(block_time);
    let rpc_cache_inner = match cache_config {
        CacheConfig::Disabled => {
            warn!(
                chain = ?chain_config.chain,
                "Cache disabled. Disabling cache."
            );
            return None;
        }
        CacheConfig::Local(config) => RpcCacheInner::Local(LocalCache::new(config.capacity)),
        CacheConfig::Redis(config) => {
            let pool = match RedisCache::pool_from_config(config).await {
                Ok(pool) => pool,
                Err(err) => {
                    error!(error = ?err, "Failed to connect to Redis cache");
                    return None;
                }
            };
            let pool = Arc::new(pool);

            RpcCacheInner::Redis(RedisCache::new(
                pool,
                chain_config.chain.id(),
                config.key_prefix.clone(),
            ))
        }
    };
    let rpc_cache = RpcCache {
        inner: rpc_cache_inner,
        ttl_manager,
    };
    Some(rpc_cache)
}

#[derive(Debug)]
pub struct RpcCache {
    inner: RpcCacheInner,
    pub ttl_manager: TTLManager,
}

impl RpcCache {
    pub fn get_ttl(&self, req: &EthRequest) -> Option<Duration> {
        self.ttl_manager.get_ttl(req)
    }

    pub async fn get(&self, req: &EthRequest) -> Option<serde_json::Value> {
        match &self.inner {
            RpcCacheInner::Local(local_cache) => local_cache.get(req).await,
            RpcCacheInner::Redis(redis_cache) => redis_cache.get(req).await,
        }
    }

    pub async fn insert(&self, req: &EthRequest, response: &serde_json::Value, ttl: Duration) {
        match &self.inner {
            RpcCacheInner::Local(local_cache) => local_cache.insert(req, response, ttl).await,
            RpcCacheInner::Redis(redis_cache) => redis_cache.insert(req, response, ttl).await,
        }
    }
}

#[derive(Debug)]
pub enum RpcCacheInner {
    Local(LocalCache),
    Redis(RedisCache),
}
