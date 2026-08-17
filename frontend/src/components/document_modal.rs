// src/components/document_modal.rs

use crate::hooks::use_documents::UseDocumentsHandle;
use crate::hooks::use_modal::ModalType;
use pi_brain_shared::{CreateDocumentRequest, Document, UpdateDocumentRequest};
use yew::prelude::*;

/// Confirmation message shown before deleting a document.
pub const DELETE_CONFIRM: &str = "Delete this document? This cannot be undone.";

/// Parse a markdown string into HTML using pulldown-cmark.
/// Tables and strikethrough are enabled for richer rendering.
fn render_markdown(content: &str) -> String {
    use pulldown_cmark::{Parser, Options, html};
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let parser = Parser::new_ext(content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

#[derive(Properties, Clone)]
pub struct DocumentModalProps {
    pub modal_type: Option<ModalType>,
    pub documents_handle: UseDocumentsHandle,
    pub on_close: Callback<()>,
    pub current_document: Option<Document>,
    pub on_edit_document: Callback<uuid::Uuid>,
    pub on_delete_document: Callback<uuid::Uuid>,
}

impl PartialEq for DocumentModalProps {
    fn eq(&self, other: &Self) -> bool {
        self.modal_type == other.modal_type && self.current_document == other.current_document
    }
}

#[function_component(DocumentModal)]
pub fn document_modal(props: &DocumentModalProps) -> Html {
    let title = use_state(|| String::new());
    let content = use_state(|| String::new());
    let tags = use_state(|| String::new());

    // Reset form when modal type changes
    let modal_type_for_closure = props.modal_type.clone();
    let current_document_for_effect = props.current_document.clone();

    use_effect_with(props.modal_type.clone(), {
        let title = title.clone();
        let content = content.clone();
        let tags = tags.clone();
        let modal_type_for_closure = modal_type_for_closure.clone();

        move |_| {
            if let Some(ModalType::EditDocument(_)) = modal_type_for_closure {
                if let Some(doc) = &current_document_for_effect {
                    title.set(doc.title.clone());
                    content.set(doc.content.clone());
                    tags.set(doc.tags.join(", "));
                }
            } else {
                title.set(String::new());
                content.set(String::new());
                tags.set(String::new());
            }
            || ()
        }
    });

    let on_title_input = {
        let title = title.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            title.set(input.value());
        })
    };

    let on_content_input = {
        let content = content.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            content.set(input.value());
        })
    };

    let on_tags_input = {
        let tags = tags.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            tags.set(input.value());
        })
    };

    let on_submit = {
        let title = title.clone();
        let content = content.clone();
        let tags = tags.clone();
        let modal_type = props.modal_type.clone();
        let documents_handle = props.documents_handle.clone();
        let on_close = props.on_close.clone();
        let current_document = props.current_document.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let tags_list: Vec<String> = (*tags)
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            match modal_type {
                Some(ModalType::CreateDocument) => {
                    let request = CreateDocumentRequest {
                        title: (*title).clone(),
                        content: (*content).clone(),
                        tags: tags_list,
                        metadata: None,
                    };
                    documents_handle.create_document.emit(request);
                    on_close.emit(());
                }
                Some(ModalType::EditDocument(id)) => {
                    let request = UpdateDocumentRequest {
                        title: Some((*title).clone()),
                        content: Some((*content).clone()),
                        tags: Some(tags_list),
                        metadata: current_document.as_ref().and_then(|d| d.metadata.clone()),
                    };
                    documents_handle.update_document.emit((id, request));
                    on_close.emit(());
                }
                _ => {}
            }
        })
    };

    let on_close_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_: yew::MouseEvent| {
            on_close.emit(());
        })
    };

    let on_close_click_clone = on_close_click.clone();

    let on_edit_click = {
        let on_edit_document = props.on_edit_document.clone();
        let current_document = props.current_document.clone();

        Callback::from(move |_: yew::MouseEvent| {
            if let Some(doc) = &current_document {
                on_edit_document.emit(doc.id);
            }
        })
    };

    let on_delete_click = {
        let on_delete_document = props.on_delete_document.clone();
        let current_document = props.current_document.clone();

        Callback::from(move |_: yew::MouseEvent| {
            if let Some(doc) = &current_document {
                on_delete_document.emit(doc.id);
            }
        })
    };

    let modal_title = match &props.modal_type {
        Some(ModalType::CreateDocument) => "Create Document",
        Some(ModalType::EditDocument(_)) => "Edit Document",
        Some(ModalType::ViewDocument(_)) => "View Document",
        None => "Document",
    };

    let is_read_only = matches!(props.modal_type, Some(ModalType::ViewDocument(_)));

    if props.modal_type.is_none() {
        return html! {};
    }

    html! {
        <div class="modal-overlay">
            <div class="modal">
                <div class="modal-header">
                    <h2 class="modal-title">{modal_title}</h2>
                    <button class="close-btn" onclick={on_close_click}>{"×"}</button>
                </div>

                if is_read_only {
                    if let Some(doc) = &props.current_document {
                        <div>
                            <div class="form-group">
                                <label>{"Title"}</label>
                                <div>{&doc.title}</div>
                            </div>
                            <div class="form-group">
                                <label>{"Content"}</label>
                                <div class="markdown-content">{Html::from_html_unchecked(AttrValue::from(render_markdown(&doc.content)))}</div>
                            </div>
                            <div class="form-group">
                                <label>{"Tags"}</label>
                                <div>
                                    {for doc.tags.iter().map(|tag| {
                                        html! { <span class="tag">{tag}</span> }
                                    })}
                                </div>
                            </div>
                            <div class="modal-actions">
                                <button class="btn btn-danger" onclick={on_delete_click}>{"Delete"}</button>
                                <button class="btn btn-secondary" onclick={on_close_click_clone.clone()}>{"Close"}</button>
                                <button class="btn btn-primary" onclick={on_edit_click}>{"Edit"}</button>
                            </div>
                        </div>
                    } else {
                        <div>{"Document not found"}</div>
                    }
                } else {
                    <form onsubmit={on_submit}>
                        <div class="form-group">
                            <label for="title">{"Title"}</label>
                            <input
                                type="text"
                                id="title"
                                value={(*title).clone()}
                                oninput={on_title_input}
                                required=true
                            />
                        </div>
                        <div class="form-group">
                            <label for="content">{"Content"}</label>
                            <textarea
                                id="content"
                                value={(*content).clone()}
                                oninput={on_content_input}
                                required=true
                            />
                        </div>
                        <div class="form-group">
                            <label for="tags">{"Tags (comma separated)"}</label>
                            <input
                                type="text"
                                id="tags"
                                value={(*tags).clone()}
                                oninput={on_tags_input}
                                placeholder="tag1, tag2, tag3"
                            />
                        </div>
                        <div class="modal-actions">
                            <button type="button" class="btn btn-secondary" onclick={on_close_click_clone.clone()}>{"Cancel"}</button>
                            <button type="submit" class="btn btn-primary">{"Save"}</button>
                        </div>
                    </form>
                }
            </div>
        </div>
    }
}
