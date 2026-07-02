// src/components/error_message.rs

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ErrorMessageProps {
    pub message: String,
}

#[function_component(ErrorMessage)]
pub fn error_message(props: &ErrorMessageProps) -> Html {
    html! {
        <div class="error">
            {&props.message}
        </div>
    }
}
