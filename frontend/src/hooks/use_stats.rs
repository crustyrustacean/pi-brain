// src/hooks/use_stats.rs

use crate::api::ApiClient;
use pi_brain_shared::PiBrainStats;
use std::sync::Arc;
use yew::prelude::*;

#[derive(Clone, Default)]
pub struct StatsState {
    pub stats: Option<PiBrainStats>,
    pub loading: bool,
    pub error: Option<String>,
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

                let result = client.get_stats().await;
                let mut next = (*state).clone();
                next.loading = false;
                match result {
                    Ok(stats) => {
                        next.stats = Some(stats);
                        next.error = None;
                    }
                    Err(e) => next.error = Some(e.to_string()),
                }
                state.set(next);
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
