// src/api/mod.rs

use gloo_net::http::{Request, Response};
use pi_brain_shared::*;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    pub fn default() -> Self {
        Self::new(String::new())
    }

    fn url(&self, endpoint: &str) -> String {
        format!("{}{}", self.base_url, endpoint)
    }

    /// Send a request and decode a JSON body, mapping non-2xx to `ApiError::Http`.
    async fn decode<T: for<'de> Deserialize<'de>>(
        response: Response,
    ) -> Result<T, ApiError> {
        if !response.ok() {
            return Err(ApiError::Http(response.status(), response.status_text()));
        }
        response
            .json::<T>()
            .await
            .map_err(|e| ApiError::Parse(e.to_string()))
    }

    // ---- Documents ----

    pub async fn list_documents(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<DocumentListResponse, ApiError> {
        let response = Request::get(&self.url(&format!(
            "/kb/documents?limit={}&offset={}",
            limit, offset
        )))
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

        Self::decode(response).await
    }

    #[allow(dead_code)]
    pub async fn get_document(&self, id: &uuid::Uuid) -> Result<Document, ApiError> {
        let response = Request::get(&self.url(&format!("/kb/documents/{}", id)))
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        Self::decode(response).await
    }

    pub async fn create_document(
        &self,
        req: &CreateDocumentRequest,
    ) -> Result<Document, ApiError> {
        let response = Request::post(&self.url("/kb/documents"))
            .json(req)
            .map_err(|e| ApiError::Network(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        Self::decode(response).await
    }

    pub async fn update_document(
        &self,
        id: &uuid::Uuid,
        req: &UpdateDocumentRequest,
    ) -> Result<Document, ApiError> {
        let response = Request::put(&self.url(&format!("/kb/documents/{}", id)))
            .json(req)
            .map_err(|e| ApiError::Network(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        Self::decode(response).await
    }

    pub async fn delete_document(&self, id: &uuid::Uuid) -> Result<(), ApiError> {
        let response = Request::delete(&self.url(&format!("/kb/documents/{}", id)))
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if !response.ok() {
            return Err(ApiError::Http(response.status(), response.status_text()));
        }
        Ok(())
    }

    // ---- Search ----

    pub async fn search_documents(
        &self,
        req: &SearchRequest,
    ) -> Result<SearchResponse, ApiError> {
        let response = Request::post(&self.url("/kb/search"))
            .json(req)
            .map_err(|e| ApiError::Network(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        Self::decode(response).await
    }

    // ---- Stats ----

    pub async fn get_stats(&self) -> Result<PiBrainStats, ApiError> {
        let response = Request::get(&self.url("/kb/stats"))
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        Self::decode(response).await
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    Network(String),
    Http(u16, String),
    Parse(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Network(msg) => write!(f, "Network error: {}", msg),
            ApiError::Http(status, msg) => write!(f, "HTTP error: {} {}", status, msg),
            ApiError::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}
