// src/components/search_bar.rs

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SearchBarProps {
    pub on_search: Callback<String>,
}

#[function_component(SearchBar)]
pub fn search_bar(props: &SearchBarProps) -> Html {
    let query = use_state(|| String::new());

    let on_input = {
        let query = query.clone();

        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            query.set(input.value());
        })
    };

    let on_submit = {
        let query = query.clone();
        let on_search = props.on_search.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let search_query = (*query).clone();
            if !search_query.trim().is_empty() {
                on_search.emit(search_query);
            }
        })
    };

    html! {
        <form class="search-bar" onsubmit={on_submit}>
            <input
                type="text"
                placeholder="Search documents..."
                value={(*query).clone()}
                oninput={on_input}
            />
            <button type="submit" class="btn btn-primary">{"Search"}</button>
        </form>
    }
}
