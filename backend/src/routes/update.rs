// src/routes/update.rs

// dependencies
use crate::database::DatabaseBackend;
use crate::domain::UpdateDocumentRequest;
use crate::utils::{e400, e500};
use actix_web::HttpResponse;
use actix_web::web::{Data, Json, Path};
use uuid::Uuid;

/// PUT /kb/documents/{id} — update an existing document. Only supplied fields change.
#[actix_web::put("/kb/documents/{id}")]
pub async fn update_document(
    path: Path<String>,
    database: Data<Box<dyn DatabaseBackend>>,
    request: Json<UpdateDocumentRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;
    let document = database
        .update_document(
            id,
            request.title.as_deref(),
            request.content.as_deref(),
            request.tags.as_deref(),
            request.metadata.as_ref(),
        )
        .await
        .map_err(e500)?;

    Ok(HttpResponse::Ok().json(document))
}
