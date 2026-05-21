// src/main.rs

mod api;
mod hooks;
mod components;
mod pages;

use yew::prelude::*;
use pages::HomePage;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <HomePage />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}