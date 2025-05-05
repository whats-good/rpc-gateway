use std::{sync::Arc, time::Duration};

use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, RedisError};
use rpc_gateway_config::RedisCacheConfig;
use tracing::error;

#[derive(Debug)]
pub struct RedisCache {
    pool: Arc<Pool<RedisConnectionManager>>,
    /// The latest block number for this chain
    chain_id: u64,
    key_prefix: Option<String>,
}

impl RedisCache {
    pub fn new(
        pool: Arc<Pool<RedisConnectionManager>>,
        chain_id: u64,
        key_prefix: Option<String>,
    ) -> Self {
        Self {
            pool,
            chain_id,
            key_prefix,
        }
    }

    pub async fn pool_from_config(
        config: &RedisCacheConfig,
    ) -> Result<Pool<RedisConnectionManager>, RedisError> {
        let manager = RedisConnectionManager::new(config.url.clone())?;
        let pool = Pool::builder()
            .max_size(config.pool_size)
            .build(manager)
            .await?;
        Ok(pool)
    }

    #[inline]
    fn key(&self, key: &str) -> String {
        format!("{}:{}", self.chain_id, key)
    }

    pub async fn get(&self, key: &str) -> Option<simd_json::OwnedValue> {
        let key = self.key(key);
        let mut con = match self.pool.get().await {
            Ok(con) => con,
            Err(err) => {
                error!(error = ?err, "Failed to establish Redis connection");
                return None;
            }
        };

        let value: Result<Option<Vec<u8>>, _> = con.get(&key).await;
        let simd_value: Option<Result<simd_json::OwnedValue, simd_json::Error>> = match value {
            Ok(value) => value.map(|mut v| simd_json::from_slice(&mut v)),
            Err(e) => {
                error!(
                    error = ?e,
                    key = ?key,
                    "Redis error",
                );
                return None;
            }
        };
        match simd_value {
            Some(Ok(value)) => Some(value),
            Some(Err(e)) => {
                error!(error = ?e, "Failed to deserialize Redis value");
                None
            }
            None => None,
        }
    }

    pub async fn insert(&self, key: String, response: &simd_json::OwnedValue, ttl: Duration) {
        let key = self.key(&key);
        // TODO: is there a better way to store the conneciton and reuse it?
        let mut connection = match self.pool.get().await {
            Ok(con) => con,
            Err(err) => {
                error!(
                    error = ?err,
                    "Failed to establish Redis connection"
                );
                return;
            }
        };

        let result: Result<(), _> = connection
            .set_ex(&key, simd_json::to_vec(&response).unwrap(), ttl.as_secs())
            .await;
        match result {
            Ok(_) => {}
            Err(err) => {
                error!(
                    error = ?err,
                    "Failed to store value in Redis cache"
                );
            }
        }
    }
}
