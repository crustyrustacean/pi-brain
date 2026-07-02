// src/routes/list.rs

// dependencies
use crate::database::DatabaseBackend;
use crate::domain::DocumentListResponse;
use crate::utils::e500;
use actix_web::HttpResponse;
use actix_web::web::{Data, Query};
use serde::Deserialize;

#[derive(Deserialize)]
struct ListQuery {
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    offset: Option<usize>,
}

/// GET /kb/documents — list all documents with pagination.
#[actix_web::get("/kb/documents")]
pub async fn list_documents(
    database: Data<Box<dyn DatabaseBackend>>,
    query: Query<ListQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let (documents, total) = database
        .search_documents("", None, limit, offset)
        .await
        .map_err(e500)?;

    Ok(HttpResponse::Ok().json(DocumentListResponse {
        documents,
        total: total as usize,
        limit,
        offset,
    }))
}
