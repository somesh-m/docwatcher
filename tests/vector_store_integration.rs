use docwatcher::{
    qdrant_store::VectorStore,
    semantic_store::{CacheEntry, CacheStore},
};
use qdrant_client::Qdrant;
use rig::serde_json;
use testcontainers::{
    GenericImage,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};

#[tokio::test]
async fn test_base() -> Result<(), anyhow::Error> {
    let container = GenericImage::new("qdrant/qdrant", "latest")
        .with_exposed_port(6334_u16.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Qdrant gRPC listening on"))
        .start()
        .await
        .expect("Failed to start Qdrant container");

    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(6334).await?;
    let client = Qdrant::from_url(&format!("http://{host}:{port}")).build()?;

    let store = VectorStore::new(client.clone(), String::from("test"), 5).await?;

    assert!(client.collection_exists("test").await?);

    let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];

    let entry = CacheEntry {
        key: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        input_hash: "test_input_hash".to_string(),
        output: serde_json::json!({
            "skills": ["Rust", "Tokio", "Qdrant"],
            "experience": 5
        }),
        prompt_version: "v1".to_string(),
        schema_version: "v1".to_string(),
        created_at: 1730000000,
        expires_at: None,
    };

    // Insert
    store
        .upsert("550e8400-e29b-41d4-a716-446655440000", &embedding, entry)
        .await?;

    // Query
    let results = store.query(&embedding, 10, 0.99).await?;

    assert!(!results.is_empty(), "Expected at least one result");

    assert!(
        (results[0].score - 1.0).abs() < 0.001,
        "Expected an exact vector match"
    );

    Ok(())
}
