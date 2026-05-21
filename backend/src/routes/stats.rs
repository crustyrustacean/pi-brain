// src/routes/stats.rs

use crate::db::Database;
use crate::response::ApiResponse;
use actix_web::{web, HttpResponse, Responder, ResponseError};
use sqlx::SqlitePool;

/// Get knowledge base statistics
#[actix_web::get("/kb/stats")]
async fn get_stats(db_pool: web::Data<SqlitePool>) -> impl Responder {
    let db = Database::new(db_pool.get_ref().clone());

    match db.get_stats().await {
        Ok(stats) => HttpResponse::Ok().json(ApiResponse::success(stats)),
        Err(e) => e.error_response(),
    }
}