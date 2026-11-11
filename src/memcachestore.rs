use memcache::exp::{Memcache, Ttl};
use rig::serde_json;

use crate::cache_store::{
    CacheEntry,
    CacheError::{self, StoreError},
    CacheStore,
};

pub struct MemcacheStore {
    client: Memcache,
}

impl CacheStore for MemcacheStore {
    async fn get(&self, key: String) -> Result<Option<CacheEntry>, CacheError> {
        let value: Option<String> = self.client.get(key).map_err(|e| StoreError(Box::new(e)))?;

        let Some(value) = value else {
            return Ok(None);
        };

        let entry: CacheEntry =
            serde_json::from_str(&value).map_err(|_| CacheError::InvalidPayload)?;

        Ok(Some(entry))
    }

    async fn upsert(&self, key: String, value: String) -> Result<(), CacheError> {
        self.client
            .set(key, value, Ttl::secs(0))
            .map_err(|e| StoreError(Box::new(e)))?;
        Ok(())
    }

    async fn delete(&self, key: String) -> Result<(), CacheError> {
        self.client
            .delete(key)
            .map_err(|e| StoreError(Box::new(e)))?;
        Ok(())
    }
}

impl MemcacheStore {
    pub async fn new(client: Memcache) -> Self {
        Self { client }
    }
}
