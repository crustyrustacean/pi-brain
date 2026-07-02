// src/routes/stats.rs

// dependencies
use crate::database::DatabaseBackend;
use crate::utils::e500;
use actix_web::HttpResponse;
use actix_web::web::Data;

/// GET /kb/stats — knowledge base statistics.
#[actix_web::get("/kb/stats")]
pub async fn get_stats(
    database: Data<Box<dyn DatabaseBackend>>,
) -> Result<HttpResponse, actix_web::Error> {
    let stats = database.get_stats().await.map_err(e500)?;

    Ok(HttpResponse::Ok().json(stats))
}
