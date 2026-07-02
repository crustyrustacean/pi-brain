// src/routes/create.rs

// dependencies
use crate::database::DatabaseBackend;
use crate::domain::{CreateDocumentRequest, Document};
use crate::utils::{compute_content_hash, e500};
use actix_web::HttpResponse;
use actix_web::web::{Data, Json};
use chrono::Utc;
use uuid::Uuid;

/// POST /kb/documents — create a new document.
#[actix_web::post("/kb/documents")]
pub async fn create_document(
    database: Data<Box<dyn DatabaseBackend>>,
    request: Json<CreateDocumentRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let request = request.into_inner();
    let content_hash = compute_content_hash(&request.content);

    // Deduplicate by content hash — return the existing document if present.
    if let Some(existing_row) = database
        .find_document_by_content_hash(&content_hash)
        .await
        .map_err(e500)?
    {
        let document: Document = existing_row.try_into().map_err(e500)?;
        return Ok(HttpResponse::Ok().json(document));
    }

    // Build the new document.
    let now = Utc::now();
    let document = Document {
        id: Uuid::new_v4(),
        title: request.title,
        content: request.content,
        content_hash,
        tags: request.tags,
        metadata: request.metadata,
        created_at: now,
        updated_at: now,
    };

    database.insert_document(&document).await.map_err(e500)?;

    Ok(HttpResponse::Ok().json(document))
}
