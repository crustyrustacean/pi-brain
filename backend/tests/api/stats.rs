// dependencies
use crate::helpers::spawn_app;
use pi_brain::domain::CreateDocumentRequest;

#[tokio::test]
async fn get_stats_returns_initial_zero_state() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body: serde_json::Value = client
        .get(format!("{}/pb/stats", &app.address))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Assert
    assert_eq!(body["total_documents"], 0);
    assert_eq!(body["total_links"], 0);
    assert!(body["unique_tags"].as_i64().unwrap() >= 0);
    assert!(body["database_size_bytes"].as_i64().unwrap() >= 0);
    assert!(body["last_updated"].is_string());
}

#[tokio::test]
async fn get_stats_counts_documents_correctly() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    for i in 0..3 {
        let request = CreateDocumentRequest {
            title: format!("Doc {i}"),
            content: format!("Content {i}"),
            tags: vec![format!("tag{i}")],
            metadata: None,
        };
        client
            .post(format!("{}/pb/documents", &app.address))
            .json(&request)
            .send()
            .await
            .unwrap();
    }

    // Act
    let body: serde_json::Value = client
        .get(format!("{}/pb/stats", &app.address))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Assert
    assert_eq!(body["total_documents"], 3);
    assert!(body["database_size_bytes"].as_i64().unwrap() > 0);
    assert!(body["last_updated"].is_string());
}
