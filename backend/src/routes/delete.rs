// src/routes/delete.rs

// dependencies
use crate::database::DatabaseBackend;
use crate::utils::e400;
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use uuid::Uuid;

/// DELETE /pb/documents/{id} — soft-delete a document.
#[actix_web::delete("/pb/documents/{id}")]
pub async fn delete_document(
    path: Path<String>,
    database: Data<Box<dyn DatabaseBackend>>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;
    database.delete_document(id).await?;

    Ok(HttpResponse::Ok().status(StatusCode::NO_CONTENT).finish())
}
