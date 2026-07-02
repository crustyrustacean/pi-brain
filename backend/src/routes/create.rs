// src/routes/create.rs

// dependencies
use crate::database::DatabaseBackend;
use crate::domain::CreateDocumentRequest;
use crate::utils::e500;
use actix_web::HttpResponse;
use actix_web::web::{Data, Json};

/// POST /kb/documents — create a new document.
#[actix_web::post("/kb/documents")]
pub async fn create_document(
    database: Data<Box<dyn DatabaseBackend>>,
    request: Json<CreateDocumentRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let document = database
        .create_document(
            &request.title,
            &request.content,
            &request.tags,
            request.metadata.as_ref(),
        )
        .await
        .map_err(e500)?;

    Ok(HttpResponse::Ok().json(document))
}
