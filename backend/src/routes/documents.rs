// src/routes/documents.rs

use crate::db::Database;
use crate::error::ApiError;
use crate::models::{CreateDocumentRequest, UpdateDocumentRequest};
use crate::response::ApiResponse;
use actix_web::{web, HttpResponse, Responder, ResponseError};
use sqlx::SqlitePool;
use uuid::Uuid;

/// Create a new document
#[actix_web::post("/kb/documents")]
async fn create_document(
    db_pool: web::Data<SqlitePool>,
    req: web::Json<CreateDocumentRequest>,
) -> impl Responder {
    let db = Database::new(db_pool.get_ref().clone());

    match db.create_document(
        &req.title,
        &req.content,
        &req.tags,
        req.metadata.as_ref(),
    )
    .await
    {
        Ok(document) => HttpResponse::Ok().json(ApiResponse::success(document)),
        Err(e) => e.error_response(),
    }
}

/// Get a document by ID
#[actix_web::get("/kb/documents/{id}")]
async fn get_document(
    db_pool: web::Data<SqlitePool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let db = Database::new(db_pool.get_ref().clone());

    match db.get_document(&id).await {
        Ok(Some(document)) => HttpResponse::Ok().json(ApiResponse::success(document)),
        Ok(None) => {
            let error = ApiError::NotFound(format!("Document {} not found", id));
            error.error_response()
        }
        Err(e) => e.error_response(),
    }
}

/// Update an existing document
#[actix_web::put("/kb/documents/{id}")]
async fn update_document(
    db_pool: web::Data<SqlitePool>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateDocumentRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let db = Database::new(db_pool.get_ref().clone());

    match db.update_document(
        &id,
        req.title.as_deref(),
        req.content.as_deref(),
        req.tags.as_deref(),
        req.metadata.as_ref(),
    )
    .await
    {
        Ok(Some(document)) => HttpResponse::Ok().json(ApiResponse::success(document)),
        Ok(None) => {
            let error = ApiError::NotFound(format!("Document {} not found", id));
            error.error_response()
        }
        Err(e) => e.error_response(),
    }
}

/// Delete a document (soft delete)
#[actix_web::delete("/kb/documents/{id}")]
async fn delete_document(
    db_pool: web::Data<SqlitePool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let db = Database::new(db_pool.get_ref().clone());

    match db.delete_document(&id).await {
        Ok(true) => HttpResponse::Ok().json(ApiResponse::success("Document deleted successfully")),
        Ok(false) => {
            let error = ApiError::NotFound(format!("Document {} not found", id));
            error.error_response()
        }
        Err(e) => e.error_response(),
    }
}

/// List all documents with pagination
#[actix_web::get("/kb/documents")]
async fn list_documents(
    db_pool: web::Data<SqlitePool>,
    query: web::Query<ListQuery>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    let db = Database::new(db_pool.get_ref().clone());

    match db.search_documents("", None, limit, offset).await {
        Ok((documents, total)) => HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
            "documents": documents,
            "total": total,
            "limit": limit,
            "offset": offset,
        }))),
        Err(e) => e.error_response(),
    }
}

#[derive(serde::Deserialize)]
struct ListQuery {
    limit: Option<usize>,
    offset: Option<usize>,
}