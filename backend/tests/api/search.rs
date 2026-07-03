// dependencies
use crate::helpers::spawn_app;
use pi_brain::domain::{CreateDocumentRequest, SearchRequest};

async fn seed(app: &crate::helpers::TestApp, docs: &[CreateDocumentRequest]) {
    let client = reqwest::Client::new();
    for doc in docs {
        client
            .post(format!("{}/kb/documents", app.address))
            .json(doc)
            .send()
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn search_returns_matching_documents() {
    // Arrange
    let app = spawn_app().await;
    seed(
        &app,
        &[
            CreateDocumentRequest {
                title: "Rust Programming".into(),
                content:
                    "Rust is a systems programming language focused on safety and performance."
                        .into(),
                tags: vec!["rust".into()],
                metadata: None,
            },
            CreateDocumentRequest {
                title: "JavaScript Guide".into(),
                content: "JavaScript is a dynamic programming language for web development.".into(),
                tags: vec!["javascript".into()],
                metadata: None,
            },
        ],
    )
    .await;
    let client = reqwest::Client::new();

    let request = SearchRequest {
        query: "Rust".to_string(),
        tags: None,
        limit: Some(10),
        offset: Some(0),
    };

    // Act
    let body: serde_json::Value = client
        .post(format!("{}/kb/search", &app.address))
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Assert
    assert_eq!(body["query"], "Rust");
    assert!(body["search_time_ms"].as_u64().is_some());

    let results = body["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["document"]["title"], "Rust Programming");
    assert!(!results[0]["excerpt"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn search_with_empty_query_filters_by_tags() {
    // Arrange
    let app = spawn_app().await;
    seed(
        &app,
        &[
            CreateDocumentRequest {
                title: "Backend Dev".into(),
                content: "Building backend systems".into(),
                tags: vec!["backend".into(), "rust".into()],
                metadata: None,
            },
            CreateDocumentRequest {
                title: "Frontend Dev".into(),
                content: "Building user interfaces".into(),
                tags: vec!["frontend".into(), "javascript".into()],
                metadata: None,
            },
        ],
    )
    .await;
    let client = reqwest::Client::new();

    let request = SearchRequest {
        query: "".to_string(),
        tags: Some(vec!["rust".to_string()]),
        limit: Some(10),
        offset: Some(0),
    };

    // Act
    let body: serde_json::Value = client
        .post(format!("{}/kb/search", &app.address))
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Assert — empty query returns all documents, filtered down to the tagged one.
    assert_eq!(body["query"], "");
    let results = body["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["document"]["title"], "Backend Dev");
}

#[tokio::test]
async fn search_does_not_panic_on_multibyte_content() {
    // Regression: `extract_excerpt` sliced at a fixed byte index and panicked
    // ("end byte index N is not a char boundary") when content held multibyte
    // UTF-8 characters such as box-drawing `─`. The panicked worker dropped the
    // connection, surfacing as a fetch failure to clients.
    let app = spawn_app().await;
    let wide_line = "─".repeat(60); // 60 × 3-byte chars = 180 bytes
    seed(
        &app,
        &[CreateDocumentRequest {
            title: "Multibyte doc".into(),
            content: format!("Tokio runtime notes\n{wide_line}\nend of document"),
            tags: vec!["rust".into()],
            metadata: None,
        }],
    )
    .await;
    let client = reqwest::Client::new();

    let request = SearchRequest {
        query: "Tokio".to_string(),
        tags: None,
        limit: Some(10),
        offset: Some(0),
    };

    // Act — previously this hit a 500 / connection drop from the panic.
    let response = client
        .post(format!("{}/kb/search", &app.address))
        .json(&request)
        .send()
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    // The excerpt must be valid UTF-8 (the panic site) and mention the query.
    let excerpt = results[0]["excerpt"].as_str().unwrap();
    assert!(excerpt.to_lowercase().contains("tokio"));
}

#[tokio::test]
async fn search_get_endpoint_works() {
    // Arrange
    let app = spawn_app().await;
    seed(
        &app,
        &[CreateDocumentRequest {
            title: "GET Search Test".into(),
            content: "Testing GET endpoint for search".into(),
            tags: vec!["test".into()],
            metadata: None,
        }],
    )
    .await;
    let client = reqwest::Client::new();

    // Act
    let body: serde_json::Value = client
        .get(format!("{}/kb/search?q=test&limit=10", &app.address))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Assert
    assert_eq!(body["query"], "test");
    assert!(body["results"].is_array());
}
