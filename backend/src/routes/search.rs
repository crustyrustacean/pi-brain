// src/routes/search.rs

// dependencies
use crate::database::DatabaseBackend;
use crate::domain::{SearchRequest, SearchResponse, SearchResult};
use crate::utils::e500;
use actix_web::HttpResponse;
use actix_web::web::{Data, Json, Query};
use serde::Deserialize;
use std::time::Instant;

/// Run a search and format the response. Shared by the POST and GET handlers.
async fn perform_search(
    database: &Box<dyn DatabaseBackend>,
    request: SearchRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let start = Instant::now();
    let limit = request.limit.unwrap_or(20);
    let offset = request.offset.unwrap_or(0);
    let query = request.query.clone();
    let tags = request.tags.clone();

    let (documents, total_count) = database
        .search_documents(&query, tags.as_deref(), limit, offset)
        .await
        .map_err(e500)?;
    let search_time_ms = start.elapsed().as_millis() as u64;

    let results: Vec<SearchResult> = documents
        .into_iter()
        .map(|doc| {
            let excerpt = extract_excerpt(&doc.content, &query, 150);
            SearchResult {
                relevance_score: 0.0, // FTS5 ranking not yet wired through
                document: doc,
                excerpt,
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(SearchResponse {
        results,
        total_count: total_count as usize,
        search_time_ms,
        query,
    }))
}

/// POST /pb/search — full-text search across documents.
#[actix_web::post("/pb/search")]
pub async fn search_documents(
    database: Data<Box<dyn DatabaseBackend>>,
    request: Json<SearchRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    perform_search(database.get_ref(), request.into_inner()).await
}

#[derive(Deserialize)]
struct SearchQuery {
    #[serde(default)]
    q: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    offset: Option<usize>,
}

/// GET /pb/search — full-text search via query parameters.
#[actix_web::get("/pb/search")]
pub async fn search_get(
    database: Data<Box<dyn DatabaseBackend>>,
    query: Query<SearchQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let request = SearchRequest {
        query: query.q.clone().unwrap_or_default(),
        tags: None,
        limit: query.limit,
        offset: query.offset,
    };
    perform_search(database.get_ref(), request).await
}

/// Extract a relevant excerpt from content around the first query match.
fn extract_excerpt(content: &str, query: &str, max_length: usize) -> String {
    if content.len() <= max_length {
        return content.to_string();
    }

    let query_lower = query.to_lowercase();
    let content_lower = content.to_lowercase();

    if let Some(pos) = content_lower.find(&query_lower) {
        // `pos` is a byte offset; snap both edges to UTF-8 char boundaries so
        // we never slice into the middle of a multibyte character.
        let start = content.floor_char_boundary(pos.saturating_sub(max_length / 2));
        let end = content.ceil_char_boundary((pos + query.len() + max_length / 2).min(content.len()));

        let excerpt = &content[start..end];
        let prefix = if start > 0 { "..." } else { "" };
        let suffix = if end < content.len() { "..." } else { "" };

        format!("{prefix}{excerpt}{suffix}")
    } else {
        // No match: take the leading run, ending on a char boundary.
        let end = content.ceil_char_boundary(max_length);
        format!("{}...", &content[..end])
    }
}
