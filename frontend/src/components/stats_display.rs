// src/components/stats_display.rs

use pi_brain_shared::PiBrainStats;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct StatsDisplayProps {
    pub stats: Option<PiBrainStats>,
}

impl PartialEq for StatsDisplayProps {
    fn eq(&self, other: &Self) -> bool {
        self.stats == other.stats
    }
}

#[function_component(StatsDisplay)]
pub fn stats_display(props: &StatsDisplayProps) -> Html {
    if let Some(stats) = &props.stats {
        html! {
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-value">{stats.total_documents}</div>
                    <div class="stat-label">{"Total Documents"}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">{stats.unique_tags}</div>
                    <div class="stat-label">{"Unique Tags"}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">{format!("{} KB", stats.database_size_bytes / 1024)}</div>
                    <div class="stat-label">{"Database Size"}</div>
                </div>
            </div>
        }
    } else {
        html! {
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-value">{"-"}</div>
                    <div class="stat-label">{"Total Documents"}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">{"-"}</div>
                    <div class="stat-label">{"Unique Tags"}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">{"-"}</div>
                    <div class="stat-label">{"Database Size"}</div>
                </div>
            </div>
        }
    }
}
