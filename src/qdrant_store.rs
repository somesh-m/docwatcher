use qdrant_client::{
    Qdrant,
    qdrant::{
        CreateCollectionBuilder, Distance, PointStruct, QueryPointsBuilder, UpsertPointsBuilder,
        VectorParamsBuilder,
    },
};
use rig::serde_json;

use crate::semantic_store::{VectorEntry, VectorError, VectorMatch, VectorStore};

pub struct QdrantStore {
    client: Qdrant,
    collection: String,
    dim: u64,
}

impl VectorStore for QdrantStore {
    async fn query(
        &self,
        embedding: &[f32],
        max_results: u64,
        threshold: f32,
    ) -> Result<Vec<VectorMatch>, VectorError> {
        let query_request = QueryPointsBuilder::new(self.collection.clone())
            .query(embedding.to_vec())
            .limit(max_results)
            .score_threshold(threshold)
            .with_payload(true);

        match self.client.query(query_request).await {
            Ok(result) => {
                let mut matches = Vec::with_capacity(result.result.len());

                for point in result.result {
                    let payload = serde_json::Value::Object(
                        point
                            .payload
                            .into_iter()
                            .map(|(key, value)| (key, value.into_json()))
                            .collect(),
                    );

                    let entry: VectorEntry =
                        serde_json::from_value(payload).map_err(VectorError::Serialization)?;

                    matches.push(VectorMatch {
                        entry,
                        score: point.score,
                    });
                }

                Ok(matches)
            }
            Err(err) => Err(VectorError::Qdrant(err)),
        }
    }

    async fn upsert(
        &self,
        key: &str,
        embedding: &[f32],
        entry: VectorEntry,
    ) -> Result<(), VectorError> {
        let payload = serde_json::to_value(&entry)?;

        let serde_json::Value::Object(payload) = payload else {
            return Err(VectorError::InvalidPayload);
        };

        let points = vec![PointStruct::new(key, embedding.to_vec(), payload)];

        match self
            .client
            .upsert_points(UpsertPointsBuilder::new(self.collection.clone(), points))
            .await
        {
            Ok(_result) => Ok(()),
            Err(err) => Err(VectorError::Qdrant(err)),
        }
    }

    async fn create_collection_if_not_exist(&self) -> Result<(), VectorError> {
        if !self
            .client
            .collection_exists(self.collection.clone())
            .await?
        {
            self.client
                .create_collection(
                    CreateCollectionBuilder::new(self.collection.clone())
                        .vectors_config(VectorParamsBuilder::new(self.dim, Distance::Cosine)),
                )
                .await?;
        }
        Ok(())
    }
}

impl QdrantStore {
    pub async fn new(client: Qdrant, collection: String, dim: u64) -> Result<Self, VectorError> {
        let inst = Self {
            client,
            collection,
            dim,
        };
        inst.create_collection_if_not_exist().await?;
        Ok(inst)
    }
}
