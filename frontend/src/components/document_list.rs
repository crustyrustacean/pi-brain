// src/components/document_list.rs

use yew::prelude::*;
use knowledge_base_shared::Document;
use crate::hooks::use_modal::ModalType;

#[derive(Properties, Clone)]
pub struct DocumentListProps {
    pub documents: Vec<Document>,
    pub on_document_click: Callback<uuid::Uuid>,
}

impl PartialEq for DocumentListProps {
    fn eq(&self, other: &Self) -> bool {
        self.documents == other.documents
    }
}

#[function_component(DocumentList)]
pub fn document_list(props: &DocumentListProps) -> Html {
    let on_click = {
        let on_document_click = props.on_document_click.clone();
        
        Callback::from(move |id: uuid::Uuid| {
            on_document_click.emit(id);
        })
    };
    
    html! {
        <div class="document-list">
            {for props.documents.iter().map(|doc| {
                let doc_id = doc.id;
                let onclick = {
                    let on_click = on_click.clone();
                    Callback::from(move |_| {
                        on_click.emit(doc_id);
                    })
                };
                
                html! {
                    <div class="document-item" {onclick}>
                        <div class="document-title">{&doc.title}</div>
                        <div class="document-content">
                            {doc.content.chars().take(200).collect::<String>()}
                            {if doc.content.len() > 200 { "..." } else { "" }}
                        </div>
                        <div class="document-meta">
                            <span>{format!("Updated: {}", doc.updated_at.format("%Y-%m-%d %H:%M"))}</span>
                        </div>
                        <div style="margin-top: 0.5rem;">
                            {for doc.tags.iter().map(|tag| {
                                html! {
                                    <span class="tag">{tag}</span>
                                }
                            })}
                        </div>
                    </div>
                }
            })}
        </div>
    }
}