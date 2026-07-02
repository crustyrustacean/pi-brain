// src/routes/read.rs

// dependencies
use crate::database::DatabaseBackend;
use crate::utils::{e400, e500};
use actix_web::HttpResponse;
use actix_web::web::{Data, Path};
use uuid::Uuid;

/// GET /kb/documents/{id} — get a single document by UUID.
#[actix_web::get("/kb/documents/{id}")]
pub async fn get_document(
    path: Path<String>,
    database: Data<Box<dyn DatabaseBackend>>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;
    let document = database.get_document(id).await.map_err(e500)?;

    Ok(HttpResponse::Ok().json(document))
}
