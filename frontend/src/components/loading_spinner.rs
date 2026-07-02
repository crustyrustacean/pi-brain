// src/components/loading_spinner.rs

use yew::prelude::*;

#[function_component(LoadingSpinner)]
pub fn loading_spinner() -> Html {
    html! {
        <div class="loading">
            <div>{"Loading..."}</div>
        </div>
    }
}
