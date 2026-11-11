pub trait KVStore {
    async fn get(&self, key: &str) -> Result<KVEntry, KVCacheError>;
    async fn upsert(&self, key: &str, value: KVEntry) -> Result<(), KVCacheError>;
    async fn delete(&self, key: &str) -> Result<(), KVCacheError>;
}

pub struct KVEntry {
    pub key: String,
    pub value: String,
}

pub enum KVCacheError {}
