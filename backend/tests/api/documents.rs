// tests/api/documents.rs

// dependencies
use crate::helpers::spawn_app;
use knowledge_base::models::{CreateDocumentRequest, UpdateDocumentRequest};
use serde_json::json;

#[tokio::test]
async fn create_document_returns_200() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    
    let create_request = CreateDocumentRequest {
        title: "Test Document".to_string(),
        content: "This is test content for the knowledge base.".to_string(),
        tags: vec!["test".to_string(), "sample".to_string()],
        metadata: Some(json!({"author": "test_user", "version": 1})),
    };

    // Act
    let response = client
        .post(&format!("{}/kb/documents", &app.address))
        .json(&create_request)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    let status = response.status();
    let body_text = response.text().await.expect("Failed to read response body");
    
    if !status.is_success() {
        panic!("Document creation failed with status {}: {}", status, body_text);
    }
    
    let body: serde_json::Value = serde_json::from_str(&body_text)
        .expect("Failed to deserialize response.");
    
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["title"], "Test Document");
    assert_eq!(body["data"]["content"], "This is test content for the knowledge base.");
    assert_eq!(body["data"]["tags"][0], "test");
    assert_eq!(body["data"]["tags"][1], "sample");
    assert!(body["data"]["id"].is_string());
    assert!(body["data"]["created_at"].is_string());
}

#[tokio::test]
async fn create_document_deduplicates_content() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    
    let create_request = CreateDocumentRequest {
        title: "Document 1".to_string(),
        content: "Same content for deduplication test".to_string(),
        tags: vec!["original".to_string()],
        metadata: None,
    };

    // Act - Create first document
    let response1 = client
        .post(&format!("{}/kb/documents", &app.address))
        .json(&create_request)
        .send()
        .await
        .expect("Failed to execute request.");

    let body1: serde_json::Value = response1.json().await.unwrap();
    let first_id = body1["data"]["id"].as_str().unwrap();

    // Create second document with same content but different title
    let mut second_request = create_request.clone();
    second_request.title = "Document 2".to_string();
    second_request.tags = vec!["duplicate".to_string()];
    
    let response2 = client
        .post(&format!("{}/kb/documents", &app.address))
        .json(&second_request)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert - Should return the same document (deduplicated)
    let body2: serde_json::Value = response2.json().await.unwrap();
    assert_eq!(body2["data"]["id"].as_str().unwrap(), first_id);
    assert_eq!(body2["data"]["title"], "Document 1"); // Original title preserved
}

#[tokio::test]
async fn get_document_returns_404_for_nonexistent() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let nonexistent_id = uuid::Uuid::new_v4();

    // Act
    let response = client
        .get(&format!("{}/kb/documents/{}", &app.address, nonexistent_id))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn get_document_returns_document_for_valid_id() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    
    let create_request = CreateDocumentRequest {
        title: "Retrieve Test".to_string(),
        content: "Content to be retrieved".to_string(),
        tags: vec!["retrieve".to_string()],
        metadata: None,
    };

    let create_response = client
        .post(&format!("{}/kb/documents", &app.address))
        .json(&create_request)
        .send()
        .await
        .expect("Failed to execute request.");

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let document_id = create_body["data"]["id"].as_str().unwrap();

    // Act
    let get_response = client
        .get(&format!("{}/kb/documents/{}", &app.address, document_id))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(get_response.status().is_success());
    
    let get_body: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(get_body["data"]["id"].as_str().unwrap(), document_id);
    assert_eq!(get_body["data"]["title"], "Retrieve Test");
}

#[tokio::test]
async fn update_document_modifies_existing_document() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    
    let create_request = CreateDocumentRequest {
        title: "Original Title".to_string(),
        content: "Original content".to_string(),
        tags: vec!["original".to_string()],
        metadata: None,
    };

    let create_response = client
        .post(&format!("{}/kb/documents", &app.address))
        .json(&create_request)
        .send()
        .await
        .expect("Failed to execute request.");

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let document_id = create_body["data"]["id"].as_str().unwrap();

    let update_request = UpdateDocumentRequest {
        title: Some("Updated Title".to_string()),
        content: Some("Updated content".to_string()),
        tags: Some(vec!["updated".to_string()]),
        metadata: Some(json!({"version": 2})),
    };

    // Act
    let update_response = client
        .put(&format!("{}/kb/documents/{}", &app.address, document_id))
        .json(&update_request)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(update_response.status().is_success());
    
    let update_body: serde_json::Value = update_response.json().await.unwrap();
    assert_eq!(update_body["data"]["title"], "Updated Title");
    assert_eq!(update_body["data"]["content"], "Updated content");
    assert_eq!(update_body["data"]["tags"][0], "updated");
    assert_eq!(update_body["data"]["metadata"]["version"], 2);
    assert!(update_body["data"]["updated_at"].as_str().unwrap() 
            != create_body["data"]["updated_at"].as_str().unwrap());
}

#[tokio::test]
async fn update_document_returns_404_for_nonexistent() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let nonexistent_id = uuid::Uuid::new_v4();

    let update_request = UpdateDocumentRequest {
        title: Some("New Title".to_string()),
        content: None,
        tags: None,
        metadata: None,
    };

    // Act
    let response = client
        .put(&format!("{}/kb/documents/{}", &app.address, nonexistent_id))
        .json(&update_request)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn delete_document_soft_deletes_document() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    
    let create_request = CreateDocumentRequest {
        title: "To Delete".to_string(),
        content: "This will be deleted".to_string(),
        tags: vec!["deleteme".to_string()],
        metadata: None,
    };

    let create_response = client
        .post(&format!("{}/kb/documents", &app.address))
        .json(&create_request)
        .send()
        .await
        .expect("Failed to execute request.");

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let document_id = create_body["data"]["id"].as_str().unwrap();

    // Act
    let delete_response = client
        .delete(&format!("{}/kb/documents/{}", &app.address, document_id))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert - Delete should succeed
    assert!(delete_response.status().is_success());
    
    // Verify document is no longer accessible
    let get_response = client
        .get(&format!("{}/kb/documents/{}", &app.address, document_id))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(get_response.status(), 404);
}

#[tokio::test]
async fn delete_document_returns_404_for_nonexistent() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let nonexistent_id = uuid::Uuid::new_v4();

    // Act
    let response = client
        .delete(&format!("{}/kb/documents/{}", &app.address, nonexistent_id))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn list_documents_returns_paginated_results() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    
    // Create multiple documents
    for i in 0..5 {
        let create_request = CreateDocumentRequest {
            title: format!("Document {}", i),
            content: format!("Content for document {}", i),
            tags: vec![format!("tag{}", i)],
            metadata: None,
        };

        client
            .post(&format!("{}/kb/documents", &app.address))
            .json(&create_request)
            .send()
            .await
            .expect("Failed to execute request.");
    }

    // Act
    let response = client
        .get(&format!("{}/kb/documents?limit=2&offset=0", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"]["limit"], 2);
    assert_eq!(body["data"]["offset"], 0);
    assert_eq!(body["data"]["total"], 5);
    assert_eq!(body["data"]["documents"].as_array().unwrap().len(), 2);
}