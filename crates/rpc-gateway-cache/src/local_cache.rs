use std::time::{Duration, Instant};

use anvil_core::eth::EthRequest;
use moka::{Expiry, future::Cache};

/// Represents a cache entry
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The actual value stored in the cache
    pub value: serde_json::Value,
    /// Duration after which this entry should expire
    pub ttl: Duration,
}

impl CacheEntry {
    /// Creates a new cache entry with the given value and TTL
    pub fn new(value: serde_json::Value, ttl: Duration) -> Self {
        Self { value, ttl }
    }
}

/// An expiry policy that uses the TTL from the cache entry
#[derive(Debug)]
pub struct TtlExpiry;

impl Expiry<String, CacheEntry> for TtlExpiry {
    fn expire_after_create(
        &self,
        _key: &String,
        value: &CacheEntry,
        _: Instant,
    ) -> Option<Duration> {
        Some(value.ttl)
    }

    fn expire_after_update(
        &self,
        key: &String,
        value: &CacheEntry,
        updated_at: Instant,
        _: Option<Duration>,
    ) -> Option<Duration> {
        self.expire_after_create(key, value, updated_at)
    }
}

/// A cache implementation with field-level TTL
#[derive(Debug)]
pub struct LocalCache {
    /// The underlying cache implementation
    cache: Cache<String, CacheEntry>,
}

impl LocalCache {
    /// Creates a new cache with the given maximum capacity and block time
    pub fn new(max_capacity: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .expire_after(TtlExpiry)
            .build();
        Self { cache }
    }
}

impl LocalCache {
    fn get_key(&self, req: &EthRequest) -> String {
        // let mut hasher = DefaultHasher::new();
        // req.hash(&mut hasher);
        // hasher.finish().to_string()
        serde_json::to_string(&req).unwrap() // TODO: is this the right way to do this?
        // TODO: may avoid saving the entire request as the key.
    }

    pub async fn get(&self, req: &EthRequest) -> Option<serde_json::Value> {
        let key = self.get_key(req);
        self.cache.get(&key).await.map(|entry| entry.value)
    }

    pub async fn insert(&self, req: &EthRequest, response: &serde_json::Value, ttl: Duration) {
        let key = self.get_key(req);
        let entry = CacheEntry::new(response.clone(), ttl);
        self.cache.insert(key, entry).await;
    }
}
