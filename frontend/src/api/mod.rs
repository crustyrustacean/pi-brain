// src/api/mod.rs

use gloo_net::http::Request;
use pi_brain_shared::*;
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, RequestMode, Response, Window};

#[derive(Debug, Clone)]
pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
    
    pub fn default() -> Self {
        Self::new("".to_string())
    }
    
    async fn fetch_json<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        method: &str,
        body: Option<String>,
    ) -> Result<T, ApiError> {
        let window = web_sys::window().expect("no global `window` exists");
        let mut opts = RequestInit::new();
        opts.method(method);
        
        if let Some(body) = body {
            opts.body(Some(&JsValue::from_str(&body)));
        }
        
        let headers = web_sys::Headers::new().unwrap();
        headers.append("Content-Type", "application/json").unwrap();
        opts.headers(&headers);
        
        let url = format!("{}{}", self.base_url, endpoint);
        let request = web_sys::Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| ApiError::RequestError(format!("{:?}", e)))?;
        
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| ApiError::NetworkError(format!("{:?}", e)))?;
        
        let response: Response = resp_value
            .dyn_into()
            .map_err(|e| ApiError::ResponseError(format!("{:?}", e)))?;
        
        if !response.ok() {
            let status = response.status();
            let status_text = response.status_text();
            return Err(ApiError::HttpError(status, status_text));
        }
        
        let text = JsFuture::from(response.text().unwrap())
            .await
            .map_err(|e| ApiError::ResponseError(format!("{:?}", e)))?;
        
        let text_string = text.as_string().unwrap_or_default();
        serde_json::from_str(&text_string).map_err(|e| ApiError::ParseError(e.to_string()))
    }
    
    // Documents API
    pub async fn list_documents(&self, limit: usize, offset: usize) -> Result<ApiResponse<DocumentListResponse>, ApiError> {
        self.fetch_json(&format!("/kb/documents?limit={}&offset={}", limit, offset), "GET", None).await
    }
    
    pub async fn get_document(&self, id: &uuid::Uuid) -> Result<ApiResponse<Document>, ApiError> {
        self.fetch_json(&format!("/kb/documents/{}", id), "GET", None).await
    }
    
    pub async fn create_document(&self, req: &CreateDocumentRequest) -> Result<ApiResponse<Document>, ApiError> {
        let body = serde_json::to_string(req).map_err(|e| ApiError::SerializeError(e.to_string()))?;
        self.fetch_json("/kb/documents", "POST", Some(body)).await
    }
    
    pub async fn update_document(&self, id: &uuid::Uuid, req: &UpdateDocumentRequest) -> Result<ApiResponse<Document>, ApiError> {
        let body = serde_json::to_string(req).map_err(|e| ApiError::SerializeError(e.to_string()))?;
        self.fetch_json(&format!("/kb/documents/{}", id), "PUT", Some(body)).await
    }
    
    pub async fn delete_document(&self, id: &uuid::Uuid) -> Result<ApiResponse<String>, ApiError> {
        self.fetch_json(&format!("/kb/documents/{}", id), "DELETE", None).await
    }
    
    // Search API
    pub async fn search_documents(&self, req: &SearchRequest) -> Result<ApiResponse<SearchResponse>, ApiError> {
        let body = serde_json::to_string(req).map_err(|e| ApiError::SerializeError(e.to_string()))?;
        self.fetch_json("/kb/search", "POST", Some(body)).await
    }
    
    // Stats API
    pub async fn get_stats(&self) -> Result<ApiResponse<KnowledgeBaseStats>, ApiError> {
        self.fetch_json("/kb/stats", "GET", None).await
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    NetworkError(String),
    RequestError(String),
    ResponseError(String),
    HttpError(u16, String),
    ParseError(String),
    SerializeError(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiError::RequestError(msg) => write!(f, "Request error: {}", msg),
            ApiError::ResponseError(msg) => write!(f, "Response error: {}", msg),
            ApiError::HttpError(status, msg) => write!(f, "HTTP error: {} {}", status, msg),
            ApiError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ApiError::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
        }
    }
}