// src/pages/home_page.rs

use yew::prelude::*;
use crate::hooks::{use_documents, use_search, use_stats, use_modal};
use crate::components::*;
use crate::hooks::use_modal::ModalType;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let documents_handle = use_documents();
    let search_handle = use_search();
    let stats_handle = use_stats();
    let modal_handle = use_modal();
    
    let current_document = use_state(|| None);
    
    // Load initial data
    use_effect_with((), {
        let documents_handle = documents_handle.clone();
        let stats_handle = stats_handle.clone();
        
        move |_| {
            documents_handle.load_documents.emit((50, 0));
            stats_handle.load_stats.emit(());
            || ()
        }
    });
    
    let on_search = {
        let search_handle = search_handle.clone();
        
        Callback::from(move |query: String| {
            let request = pi_brain_shared::SearchRequest {
                query: query.clone(),
                tags: None,
                limit: Some(50),
                offset: Some(0),
            };
            search_handle.search.emit(request);
        })
    };
    
    let on_document_click = {
        let modal_handle = modal_handle.clone();
        let current_document = current_document.clone();
        let documents = documents_handle.state.documents.clone();
        
        Callback::from(move |id: uuid::Uuid| {
            if let Some(doc) = documents.iter().find(|d| d.id == id) {
                current_document.set(Some(doc.clone()));
                modal_handle.open_modal.emit(ModalType::ViewDocument(id));
            }
        })
    };
    
    let on_create_document = {
        let modal_handle = modal_handle.clone();
        
        Callback::from(move |_| {
            modal_handle.open_modal.emit(ModalType::CreateDocument);
        })
    };
    
    let on_edit_document = {
        let modal_handle = modal_handle.clone();
        let current_document = current_document.clone();
        
        Callback::from(move |_: yew::MouseEvent| {
            if let Some(doc) = (*current_document).clone() {
                modal_handle.open_modal.emit(ModalType::EditDocument(doc.id));
            }
        })
    };
    
    let on_delete_document = {
        let documents_handle = documents_handle.clone();
        let modal_handle = modal_handle.clone();
        let current_document = current_document.clone();
        
        Callback::from(move |_: yew::MouseEvent| {
            if let Some(_doc) = (*current_document).clone() {
                let confirmed = web_sys::window()
                    .expect("no global `window` exists")
                    .confirm()
                    .unwrap_or(false);
                
                if confirmed {
                    if let Some(doc) = (*current_document).clone() {
                        documents_handle.delete_document.emit(doc.id);
                    }
                    modal_handle.close_modal.emit(());
                }
            }
        })
    };
    
    let on_close_modal = {
        let modal_handle = modal_handle.clone();
        let current_document = current_document.clone();
        
        Callback::from(move |_| {
            modal_handle.close_modal.emit(());
            current_document.set(None);
        })
    };
    
    let display_documents = if search_handle.state.results.results.is_empty() {
        &documents_handle.state.documents
    } else {
        &search_handle.state.results.results.iter().map(|r| r.document.clone()).collect::<Vec<_>>()
    };
    
    html! {
        <>
            <header>
                <div class="container">
                    <h1>{"pi-brain"}</h1>
                </div>
            </header>
            
            <main class="container">
                // Stats Display
                <StatsDisplay stats={stats_handle.state.stats.clone()} />
                
                // Search Bar
                <SearchBar on_search={on_search.clone()} />
                
                // Action Buttons
                <div style="margin-bottom: 1rem;">
                    <button class="btn btn-primary" onclick={on_create_document}>
                        {"+ New Document"}
                    </button>
                    if !search_handle.state.results.results.is_empty() {
                        <button class="btn btn-secondary" onclick={Callback::from(move |_| {
                            search_handle.clear_search.emit(());
                        })}>
                            {"Clear Search"}
                        </button>
                    }
                </div>
                
                // Error Display
                if let Some(error) = &documents_handle.state.error {
                    <ErrorMessage message={error.clone()} />
                }
                
                if let Some(error) = &search_handle.state.error {
                    <ErrorMessage message={error.clone()} />
                }
                
                // Loading State
                if documents_handle.state.loading || search_handle.state.loading {
                    <LoadingSpinner />
                }
                
                // Document List
                <DocumentList 
                    documents={display_documents.clone()}
                    on_document_click={on_document_click.clone()}
                />
                
                // Empty State
                if display_documents.is_empty() && !documents_handle.state.loading && !search_handle.state.loading {
                    <div class="card" style="text-align: center; padding: 2rem;">
                        <h3>{"No documents found"}</h3>
                        <p style="color: #666; margin-top: 0.5rem;">
                            {"Create your first document to get started!"}
                        </p>
                    </div>
                }
            </main>
            
            // Document Modal
            <DocumentModal 
                modal_type={modal_handle.state.modal_type.clone()}
                documents_handle={documents_handle.clone()}
                on_close={on_close_modal}
                current_document={(*current_document).clone()}
            />
        </>
    }
}