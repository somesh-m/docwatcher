use serde::{Deserialize, Serialize};

pub trait CacheStore {
    async fn get(&self, key: String) -> Result<Option<CacheEntry>, CacheError>;
    async fn upsert(&self, key: String, value: String) -> Result<(), CacheError>;
    async fn delete(&self, key: String) -> Result<(), CacheError>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub value: String,
    pub created_at: u64,
    pub expires_at: Option<u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Invalid Payload")]
    InvalidPayload,

    #[error("Vector store operation failed: {0}")]
    StoreError(#[source] Box<dyn std::error::Error + Send + Sync>),
}
