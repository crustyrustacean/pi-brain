// backend_impl.rs - Backend-specific implementations for shared types

use super::ApiResponse;
use actix_web::{HttpRequest, HttpResponse, Responder, body::BoxBody, http::header::ContentType};
use serde::Serialize;

impl<T: Serialize> Responder for ApiResponse<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        match serde_json::to_string(&self) {
            Ok(body) => HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(body),
            Err(e) => {
                tracing::error!("Failed to serialize API response: {:?}", e);
                HttpResponse::InternalServerError()
                    .content_type(ContentType::plaintext())
                    .body("Internal Server Error")
            }
        }
    }
}