// src/hooks/use_search.rs

use crate::api::ApiClient;
use pi_brain_shared::{SearchRequest, SearchResponse};
use std::sync::Arc;
use yew::prelude::*;

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

                let result = client.search_documents(&request).await;
                let mut next = (*state).clone();
                next.loading = false;
                match result {
                    Ok(results) => {
                        next.results = results;
                        next.error = None;
                    }
                    Err(e) => next.error = Some(e.to_string()),
                }
                state.set(next);
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
