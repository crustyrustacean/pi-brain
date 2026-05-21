// src/hooks/use_stats.rs

use crate::api::{ApiClient, ApiError};
use knowledge_base_shared::{KnowledgeBaseStats, ApiResponse};
use yew::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct StatsState {
    pub stats: Option<KnowledgeBaseStats>,
    pub loading: bool,
    pub error: Option<String>,
}

impl Default for StatsState {
    fn default() -> Self {
        Self {
            stats: None,
            loading: false,
            error: None,
        }
    }
}

#[hook]
pub fn use_stats() -> UseStatsHandle {
    let state = use_state(|| StatsState::default());
    let client = use_memo((), |_| Arc::new(ApiClient::default()));
    
    let load_stats = {
        let state = state.clone();
        let client = client.clone();
        
        Callback::from(move |_| {
            let state = state.clone();
            let client = client.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                state.set(StatsState {
                    loading: true,
                    error: None,
                    ..(*state).clone()
                });
                
                match client.get_stats().await {
                    Ok(ApiResponse { success: true, data: Some(stats), .. }) => {
                        state.set(StatsState {
                            stats: Some(stats),
                            loading: false,
                            error: None,
                        });
                    }
                    Ok(ApiResponse { success: false, error: Some(err), .. }) => {
                        state.set(StatsState {
                            loading: false,
                            error: Some(err),
                            ..(*state).clone()
                        });
                    }
                    Err(e) => {
                        state.set(StatsState {
                            loading: false,
                            error: Some(e.to_string()),
                            ..(*state).clone()
                        });
                    }
                    _ => {
                        state.set(StatsState {
                            loading: false,
                            error: Some("Unexpected response format".to_string()),
                            ..(*state).clone()
                        });
                    }
                }
            });
        })
    };
    
    UseStatsHandle {
        state: (*state).clone(),
        load_stats,
    }
}

#[derive(Clone)]
pub struct UseStatsHandle {
    pub state: StatsState,
    pub load_stats: Callback<()>,
}