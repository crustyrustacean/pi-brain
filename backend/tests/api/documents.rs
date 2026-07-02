// dependencies
use crate::helpers::spawn_app;
use pi_brain::domain::{CreateDocumentRequest, UpdateDocumentRequest};
use serde_json::json;
use sqlx::FromRow;

#[derive(FromRow)]
struct StoredDocument {
    id: String,
    title: String,
    content: String,
}

#[tokio::test]
async fn create_document_returns_200_with_document() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let request = CreateDocumentRequest {
        title: "Test Document".to_string(),
        content: "This is test content for the knowledge base.".to_string(),
        tags: vec!["test".to_string(), "sample".to_string()],
        metadata: Some(json!({"author": "test_user", "version": 1})),
    };

    // Act
    let response = client
        .post(format!("{}/kb/documents", &app.address))
        .json(&request)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.unwrap();
    let id = body["id"].as_str().unwrap().to_string();

    assert_eq!(body["title"], "Test Document");
    assert_eq!(body["content"], "This is test content for the knowledge base.");
    assert_eq!(body["tags"][0], "test");
    assert_eq!(body["tags"][1], "sample");
    assert_eq!(body["metadata"]["author"], "test_user");

    // Verify the row landed in the database.
    let stored: StoredDocument =
        sqlx::query_as("SELECT id, title, content FROM documents WHERE id = ?")
            .bind(&id)
            .fetch_one(app.database.pool())
            .await
            .unwrap();

    assert_eq!(stored.id, id);
    assert_eq!(stored.title, request.title);
    assert_eq!(stored.content, request.content);
}

#[tokio::test]
async fn create_document_deduplicates_content() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let request = CreateDocumentRequest {
        title: "Document 1".to_string(),
        content: "Same content for deduplication test".to_string(),
        tags: vec!["original".to_string()],
        metadata: None,
    };

    // Act — create the first document.
    let body1: serde_json::Value = client
        .post(format!("{}/kb/documents", &app.address))
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let first_id = body1["id"].as_str().unwrap();

    // Create a second document with the same content but a different title.
    let mut second = request.clone();
    second.title = "Document 2".to_string();
    second.tags = vec!["duplicate".to_string()];

    let body2: serde_json::Value = client
        .post(format!("{}/kb/documents", &app.address))
        .json(&second)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Assert — the existing document is returned unchanged.
    assert_eq!(body2["id"].as_str().unwrap(), first_id);
    assert_eq!(body2["title"], "Document 1");
}

#[tokio::test]
async fn get_document_returns_document_for_valid_id() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let request = CreateDocumentRequest {
        title: "Retrieve Test".to_string(),
        content: "Content to be retrieved".to_string(),
        tags: vec!["retrieve".to_string()],
        metadata: None,
    };
    let created: serde_json::Value = client
        .post(format!("{}/kb/documents", &app.address))
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = created["id"].as_str().unwrap();

    // Act
    let body: serde_json::Value = client
        .get(format!("{}/kb/documents/{}", &app.address, id))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Assert
    assert_eq!(body["id"].as_str().unwrap(), id);
    assert_eq!(body["title"], "Retrieve Test");
}

#[tokio::test]
async fn get_document_returns_500_for_nonexistent_id() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let missing = uuid::Uuid::new_v4();

    // Act
    let response = client
        .get(format!("{}/kb/documents/{}", &app.address, missing))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert — the repository surfaces `NotFound`, mapped to a 500 via `e500`.
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[tokio::test]
async fn update_document_applies_partial_changes() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let create = CreateDocumentRequest {
        title: "Original Title".to_string(),
        content: "Original content".to_string(),
        tags: vec!["original".to_string()],
        metadata: None,
    };
    let created: serde_json::Value = client
        .post(format!("{}/kb/documents", &app.address))
        .json(&create)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = created["id"].as_str().unwrap();

    let update = UpdateDocumentRequest {
        title: Some("Updated Title".to_string()),
        content: None,
        tags: Some(vec!["updated".to_string()]),
        metadata: Some(json!({"version": 2})),
    };

    // Act
    let body: serde_json::Value = client
        .put(format!("{}/kb/documents/{}", &app.address, id))
        .json(&update)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Assert — supplied fields change, absent fields are preserved.
    assert_eq!(body["title"], "Updated Title");
    assert_eq!(body["content"], "Original content");
    assert_eq!(body["tags"][0], "updated");
    assert_eq!(body["metadata"]["version"], 2);
}

#[tokio::test]
async fn delete_document_soft_deletes_then_hides_it() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let create = CreateDocumentRequest {
        title: "To Delete".to_string(),
        content: "This will be deleted".to_string(),
        tags: vec!["deleteme".to_string()],
        metadata: None,
    };
    let created: serde_json::Value = client
        .post(format!("{}/kb/documents", &app.address))
        .json(&create)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = created["id"].as_str().unwrap();

    // Act — soft-delete.
    let delete_response = client
        .delete(format!("{}/kb/documents/{}", &app.address, id))
        .send()
        .await
        .unwrap();
    assert!(delete_response.status().is_success());

    // Assert — the document is no longer retrievable.
    let get_response = client
        .get(format!("{}/kb/documents/{}", &app.address, id))
        .send()
        .await
        .unwrap();
    assert_eq!(
        get_response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[tokio::test]
async fn list_documents_returns_paginated_results() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    for i in 0..5 {
        let request = CreateDocumentRequest {
            title: format!("Document {i}"),
            content: format!("Content for document {i}"),
            tags: vec![format!("tag{i}")],
            metadata: None,
        };
        client
            .post(format!("{}/kb/documents", &app.address))
            .json(&request)
            .send()
            .await
            .unwrap();
    }

    // Act
    let body: serde_json::Value = client
        .get(format!("{}/kb/documents?limit=2&offset=0", &app.address))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Assert
    assert_eq!(body["limit"], 2);
    assert_eq!(body["offset"], 0);
    assert_eq!(body["total"], 5);
    assert_eq!(body["documents"].as_array().unwrap().len(), 2);
}
