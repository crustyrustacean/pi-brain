// src/routes/read.rs

// dependencies
use crate::database::DatabaseBackend;
use crate::utils::e400;
use actix_web::HttpResponse;
use actix_web::web::{Data, Path};
use pi_brain_shared::Document;
use uuid::Uuid;

/// GET /pb/documents/{id} — get a single document by UUID.
#[actix_web::get("/pb/documents/{id}")]
pub async fn get_document(
    path: Path<String>,
    database: Data<Box<dyn DatabaseBackend>>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;
    let document_row = database.get_document(id).await?;
    let document: Document = document_row.try_into().map_err(crate::utils::e500)?;

    Ok(HttpResponse::Ok().json(document))
}
