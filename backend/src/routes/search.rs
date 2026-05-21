// src/routes/search.rs

use crate::db::Database;
use crate::error::ApiError;
use crate::models::{SearchRequest, SearchResult, SearchResponse};
use crate::response::ApiResponse;
use actix_web::{web, HttpResponse, Responder, ResponseError};
use sqlx::SqlitePool;
use std::time::Instant;

/// Helper function to perform search and format response
async fn perform_search(
    db: Database,
    request: SearchRequest,
) -> Result<HttpResponse, ApiError> {
    let start = Instant::now();
    let limit = request.limit.unwrap_or(20);
    let offset = request.offset.unwrap_or(0);
    let query = request.query.clone();
    let tags = request.tags.clone();

    let (documents, total_count) = db.search_documents(&query, tags.as_deref(), limit, offset).await?;
    let search_time_ms = start.elapsed().as_millis() as u64;

    // Convert documents to search results with excerpts
    let results: Vec<SearchResult> = documents
        .into_iter()
        .map(|doc| {
            let excerpt = extract_excerpt(&doc.content, &query, 150);
            SearchResult {
                relevance_score: 0.0, // Would be calculated by FTS5 ranking
                document: doc,
                excerpt,
            }
        })
        .collect();

    let response = SearchResponse {
        results,
        total_count: total_count as usize,
        search_time_ms,
        query,
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// Full-text search across documents
#[actix_web::post("/kb/search")]
async fn search_documents(
    db_pool: web::Data<SqlitePool>,
    req: web::Json<SearchRequest>,
) -> impl Responder {
    let db = Database::new(db_pool.get_ref().clone());
    match perform_search(db, req.into_inner()).await {
        Ok(response) => response,
        Err(e) => e.error_response(),
    }
}

/// Simple full-text search via GET for easier access
#[actix_web::get("/kb/search")]
async fn search_get(
    db_pool: web::Data<SqlitePool>,
    query: web::Query<SearchQuery>,
) -> impl Responder {
    let search_request = SearchRequest {
        query: query.q.clone().unwrap_or_default(),
        tags: None,
        limit: query.limit,
        offset: query.offset,
    };

    let db = Database::new(db_pool.get_ref().clone());
    match perform_search(db, search_request).await {
        Ok(response) => response,
        Err(e) => e.error_response(),
    }
}

#[derive(serde::Deserialize)]
struct SearchQuery {
    q: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

/// Extract a relevant excerpt from content around search terms
fn extract_excerpt(content: &str, query: &str, max_length: usize) -> String {
    if content.len() <= max_length {
        return content.to_string();
    }

    let query_lower = query.to_lowercase();
    let content_lower = content.to_lowercase();

    if let Some(pos) = content_lower.find(&query_lower) {
        let start = pos.saturating_sub(max_length / 2);
        let end = (pos + query.len() + max_length / 2).min(content.len());

        let excerpt = &content[start..end];
        let prefix = if start > 0 { "..." } else { "" };
        let suffix = if end < content.len() { "..." } else { "" };

        format!("{}{}{}", prefix, excerpt, suffix)
    } else {
        format!("{}...", &content[..max_length])
    }
}