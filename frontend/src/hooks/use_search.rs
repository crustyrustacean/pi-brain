// src/hooks/use_search.rs

use crate::api::{ApiClient, ApiError};
use knowledge_base_shared::{SearchRequest, SearchResponse, ApiResponse};
use yew::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct SearchState {
    pub results: SearchResponse,
    pub loading: bool,
    pub error: Option<String>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            results: SearchResponse {
                results: Vec::new(),
                total_count: 0,
                search_time_ms: 0,
                query: String::new(),
            },
            loading: false,
            error: None,
        }
    }
}

#[hook]
pub fn use_search() -> UseSearchHandle {
    let state = use_state(|| SearchState::default());
    let client = use_memo((), |_| Arc::new(ApiClient::default()));
    
    let search = {
        let state = state.clone();
        let client = client.clone();
        
        Callback::from(move |request: SearchRequest| {
            let state = state.clone();
            let client = client.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                state.set(SearchState {
                    loading: true,
                    error: None,
                    ..(*state).clone()
                });
                
                match client.search_documents(&request).await {
                    Ok(ApiResponse { success: true, data: Some(response), .. }) => {
                        state.set(SearchState {
                            results: response,
                            loading: false,
                            error: None,
                        });
                    }
                    Ok(ApiResponse { success: false, error: Some(err), .. }) => {
                        state.set(SearchState {
                            loading: false,
                            error: Some(err),
                            ..(*state).clone()
                        });
                    }
                    Err(e) => {
                        state.set(SearchState {
                            loading: false,
                            error: Some(e.to_string()),
                            ..(*state).clone()
                        });
                    }
                    _ => {
                        state.set(SearchState {
                            loading: false,
                            error: Some("Unexpected response format".to_string()),
                            ..(*state).clone()
                        });
                    }
                }
            });
        })
    };
    
    let clear_search = {
        let state = state.clone();
        
        Callback::from(move |_| {
            state.set(SearchState::default());
        })
    };
    
    UseSearchHandle {
        state: (*state).clone(),
        search,
        clear_search,
    }
}

#[derive(Clone)]
pub struct UseSearchHandle {
    pub state: SearchState,
    pub search: Callback<SearchRequest>,
    pub clear_search: Callback<()>,
}