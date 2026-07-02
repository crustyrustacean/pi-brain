// src/hooks/use_documents.rs

use crate::api::ApiClient;
use pi_brain_shared::{
    CreateDocumentRequest, Document, DocumentListResponse, UpdateDocumentRequest,
};
use std::sync::Arc;
use yew::prelude::*;

#[derive(Clone, Default)]
pub struct DocumentsState {
    pub documents: Vec<Document>,
    pub total: usize,
    pub loading: bool,
    pub error: Option<String>,
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

                let result = client.list_documents(limit, offset).await;
                let mut next = (*state).clone();
                next.loading = false;
                match result {
                    Ok(DocumentListResponse {
                        documents, total, ..
                    }) => {
                        next.documents = documents;
                        next.total = total;
                        next.error = None;
                    }
                    Err(e) => next.error = Some(e.to_string()),
                }
                state.set(next);
            });
        })
    };

    let create_document = {
        let client = client.clone();

        Callback::from(move |request: CreateDocumentRequest| {
            let client = client.clone();

            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = client.create_document(&request).await {
                    web_sys::console::error_1(&format!("Failed to create document: {}", e).into());
                } else {
                    reload();
                }
            });
        })
    };

    let update_document = {
        let client = client.clone();

        Callback::from(move |(id, request): (uuid::Uuid, UpdateDocumentRequest)| {
            let client = client.clone();

            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = client.update_document(&id, &request).await {
                    web_sys::console::error_1(&format!("Failed to update document: {}", e).into());
                } else {
                    reload();
                }
            });
        })
    };

    let delete_document = {
        let client = client.clone();

        Callback::from(move |id: uuid::Uuid| {
            let client = client.clone();

            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = client.delete_document(&id).await {
                    web_sys::console::error_1(&format!("Failed to delete document: {}", e).into());
                } else {
                    reload();
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

fn reload() {
    web_sys::window()
        .expect("no global `window` exists")
        .location()
        .reload()
        .expect("failed to reload page");
}

#[derive(Clone)]
pub struct UseDocumentsHandle {
    pub state: DocumentsState,
    pub load_documents: Callback<(usize, usize)>,
    pub create_document: Callback<CreateDocumentRequest>,
    pub update_document: Callback<(uuid::Uuid, UpdateDocumentRequest)>,
    pub delete_document: Callback<uuid::Uuid>,
}
