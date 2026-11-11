use rig::serde_json;
use serde::{Deserialize, Serialize};

pub trait VectorStore {
    async fn query(
        &self,
        embedding: &[f32],
        max_results: u64,
        threshold: f32,
    ) -> Result<Vec<VectorMatch>, VectorError>;
    async fn upsert(
        &self,
        key: &str,
        embedding: &[f32],
        entry: VectorEntry,
    ) -> Result<(), VectorError>;
    async fn create_collection_if_not_exist(&self) -> Result<(), VectorError>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorEntry {
    pub key: String,
    pub input_hash: String,
    pub output: serde_json::Value,
    pub prompt_version: String,
    pub schema_version: String,
    pub created_at: u64,
    pub expires_at: Option<u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum VectorError {
    #[error("Qdrant operation failed: {0}")]
    Qdrant(#[from] qdrant_client::QdrantError),

    #[error("Serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid payload")]
    InvalidPayload,
}

pub struct VectorMatch {
    pub entry: VectorEntry,
    pub score: f32,
}
