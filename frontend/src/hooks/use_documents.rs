// src/hooks/use_documents.rs

use crate::api::{ApiClient, ApiError};
use knowledge_base_shared::{CreateDocumentRequest, Document, UpdateDocumentRequest, DocumentListResponse, ApiResponse};
use yew::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct DocumentsState {
    pub documents: Vec<Document>,
    pub total: usize,
    pub loading: bool,
    pub error: Option<String>,
}

impl Default for DocumentsState {
    fn default() -> Self {
        Self {
            documents: Vec::new(),
            total: 0,
            loading: false,
            error: None,
        }
    }
}

#[hook]
pub fn use_documents() -> UseDocumentsHandle {
    let state = use_state(|| DocumentsState::default());
    let client = use_memo((), |_| Arc::new(ApiClient::default()));
    
    let load_documents = {
        let state = state.clone();
        let client = client.clone();
        
        Callback::from(move |(limit, offset): (usize, usize)| {
            let state = state.clone();
            let client = client.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                state.set(DocumentsState {
                    loading: true,
                    error: None,
                    ..(*state).clone()
                });
                
                match client.list_documents(limit, offset).await {
                    Ok(ApiResponse { success: true, data: Some(response), .. }) => {
                        state.set(DocumentsState {
                            documents: response.documents,
                            total: response.total,
                            loading: false,
                            error: None,
                        });
                    }
                    Ok(ApiResponse { success: false, error: Some(err), .. }) => {
                        state.set(DocumentsState {
                            loading: false,
                            error: Some(err),
                            ..(*state).clone()
                        });
                    }
                    Err(e) => {
                        state.set(DocumentsState {
                            loading: false,
                            error: Some(e.to_string()),
                            ..(*state).clone()
                        });
                    }
                    _ => {
                        state.set(DocumentsState {
                            loading: false,
                            error: Some("Unexpected response format".to_string()),
                            ..(*state).clone()
                        });
                    }
                }
            });
        })
    };
    
    let create_document = {
        let client = client.clone();
        
        Callback::from(move |request: CreateDocumentRequest| {
            let client = client.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                match client.create_document(&request).await {
                    Ok(ApiResponse { success: true, .. }) => {
                        // Trigger reload via callback or event
                        web_sys::window()
                            .expect("no global `window` exists")
                            .location()
                            .reload()
                            .expect("failed to reload page");
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to create document: {}", e).into());
                    }
                    _ => {}
                }
            });
        })
    };
    
    let update_document = {
        let client = client.clone();
        
        Callback::from(move |(id, request): (uuid::Uuid, UpdateDocumentRequest)| {
            let client = client.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                match client.update_document(&id, &request).await {
                    Ok(ApiResponse { success: true, .. }) => {
                        web_sys::window()
                            .expect("no global `window` exists")
                            .location()
                            .reload()
                            .expect("failed to reload page");
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to update document: {}", e).into());
                    }
                    _ => {}
                }
            });
        })
    };
    
    let delete_document = {
        let client = client.clone();
        
        Callback::from(move |id: uuid::Uuid| {
            let client = client.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                match client.delete_document(&id).await {
                    Ok(ApiResponse { success: true, .. }) => {
                        web_sys::window()
                            .expect("no global `window` exists")
                            .location()
                            .reload()
                            .expect("failed to reload page");
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to delete document: {}", e).into());
                    }
                    _ => {}
                }
            });
        })
    };
    
    UseDocumentsHandle {
        state: (*state).clone(),
        load_documents,
        create_document,
        update_document,
        delete_document,
    }
}

#[derive(Clone)]
pub struct UseDocumentsHandle {
    pub state: DocumentsState,
    pub load_documents: Callback<(usize, usize)>,
    pub create_document: Callback<CreateDocumentRequest>,
    pub update_document: Callback<(uuid::Uuid, UpdateDocumentRequest)>,
    pub delete_document: Callback<uuid::Uuid>,
}