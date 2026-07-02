// src/routes/update.rs

// dependencies
use crate::database::DatabaseBackend;
use crate::domain::{Document, UpdateDocumentRequest};
use crate::utils::{compute_content_hash, e400, e500};
use actix_web::HttpResponse;
use actix_web::web::{Data, Json, Path};
use chrono::Utc;
use uuid::Uuid;

/// PUT /kb/documents/{id} — update an existing document. Only supplied fields change.
#[actix_web::put("/kb/documents/{id}")]
pub async fn update_document(
    path: Path<String>,
    database: Data<Box<dyn DatabaseBackend>>,
    request: Json<UpdateDocumentRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;

    // Fetch the existing document and lift it into the domain shape for merging.
    let existing_row = database.get_document(id).await?;
    let mut document: Document = existing_row.try_into().map_err(e500)?;

    // Merge the partial update — pure domain work, before any write.
    let UpdateDocumentRequest {
        title,
        content,
        tags,
        metadata,
    } = request.into_inner();
    let mut content_changed = false;
    if let Some(t) = title {
        document.title = t;
    }
    if let Some(c) = content {
        document.content = c;
        content_changed = true;
    }
    if let Some(t) = tags {
        document.tags = t;
    }
    if let Some(m) = metadata {
        document.metadata = Some(m);
    }

    // Recompute the content hash only when the content actually changed.
    if content_changed {
        document.content_hash = compute_content_hash(&document.content);
    }
    document.updated_at = Utc::now();

    // Persist: hand the fully-resolved document to a dumb write.
    database.update_document(&document).await.map_err(e500)?;

    Ok(HttpResponse::Ok().json(document))
}
