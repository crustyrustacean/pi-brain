// src/main.rs

mod api;
mod components;
mod hooks;
mod pages;

use pages::HomePage;
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <HomePage />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
