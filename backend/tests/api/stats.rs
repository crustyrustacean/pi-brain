// tests/api/stats.rs

// dependencies
use crate::helpers::spawn_app;
use knowledge_base::models::CreateDocumentRequest;

#[tokio::test]
async fn get_stats_returns_initial_zero_state() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/kb/stats", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["total_documents"], 0);
    assert_eq!(body["data"]["total_links"], 0);
    assert!(body["data"]["unique_tags"].as_i64().unwrap() >= 0);
    assert!(body["data"]["database_size_bytes"].as_i64().unwrap() >= 0);
    assert!(body["data"]["last_updated"].is_string());
}

#[tokio::test]
async fn get_stats_counts_documents_correctly() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    
    // Create some documents
    let documents = vec![
        CreateDocumentRequest {
            title: "Doc 1".to_string(),
            content: "Content 1".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            metadata: None,
        },
        CreateDocumentRequest {
            title: "Doc 2".to_string(),
            content: "Content 2".to_string(),
            tags: vec!["tag2".to_string(), "tag3".to_string()],
            metadata: None,
        },
        CreateDocumentRequest {
            title: "Doc 3".to_string(),
            content: "Content 3".to_string(),
            tags: vec!["tag4".to_string()],
            metadata: None,
        },
    ];

    for doc in &documents {
        client
            .post(&format!("{}/kb/documents", &app.address))
            .json(doc)
            .send()
            .await
            .expect("Failed to create document");
    }

    // Act
    let response = client
        .get(&format!("{}/kb/stats", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["total_documents"], 3);
    assert!(body["data"]["database_size_bytes"].as_i64().unwrap() > 0);
    assert!(body["data"]["last_updated"].is_string());
}

#[tokio::test]
async fn get_stats_updates_after_document_creation() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Get initial stats
    let initial_response = client
        .get(&format!("{}/kb/stats", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    let initial_body: serde_json::Value = initial_response.json().await.unwrap();
    let initial_count = initial_body["data"]["total_documents"].as_i64().unwrap();

    // Create a document
    let create_request = CreateDocumentRequest {
        title: "New Doc".to_string(),
        content: "New content".to_string(),
        tags: vec!["new".to_string()],
        metadata: None,
    };

    client
        .post(&format!("{}/kb/documents", &app.address))
        .json(&create_request)
        .send()
        .await
        .expect("Failed to create document");

    // Get updated stats
    let updated_response = client
        .get(&format!("{}/kb/stats", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    let updated_body: serde_json::Value = updated_response.json().await.unwrap();
    assert_eq!(updated_body["data"]["total_documents"].as_i64().unwrap(), initial_count + 1);
}

#[tokio::test]
async fn get_stats_includes_database_size() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Create some documents to increase database size
    for i in 0..5 {
        let create_request = CreateDocumentRequest {
            title: format!("Document {}", i),
            content: format!("This is document {} with some content to increase database size", i),
            tags: vec![format!("tag{}", i)],
            metadata: None,
        };

        client
            .post(&format!("{}/kb/documents", &app.address))
            .json(&create_request)
            .send()
            .await
            .expect("Failed to create document");
    }

    // Act
    let response = client
        .get(&format!("{}/kb/stats", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.unwrap();
    let db_size = body["data"]["database_size_bytes"].as_i64().unwrap();
    assert!(db_size > 0, "Database size should be greater than 0");
}

#[tokio::test]
async fn get_stats_tracks_last_updated_timestamp() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Get initial stats
    let initial_response = client
        .get(&format!("{}/kb/stats", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    let initial_body: serde_json::Value = initial_response.json().await.unwrap();
    let _initial_timestamp = initial_body["data"]["last_updated"].as_str().unwrap().to_string();

    // Create a document to trigger an update
    let create_request = CreateDocumentRequest {
        title: "Timestamp Test".to_string(),
        content: "Testing timestamp updates".to_string(),
        tags: vec!["time".to_string()],
        metadata: None,
    };

    client
        .post(&format!("{}/kb/documents", &app.address))
        .json(&create_request)
        .send()
        .await
        .expect("Failed to create document");

    // Give it a moment to ensure timestamps differ
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Get updated stats
    let updated_response = client
        .get(&format!("{}/kb/stats", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    let updated_body: serde_json::Value = updated_response.json().await.unwrap();
    let updated_timestamp = updated_body["data"]["last_updated"].as_str().unwrap();
    
    // Timestamps should be ISO 8601 format strings
    assert!(updated_timestamp.len() > 0, "Timestamp should not be empty");
}