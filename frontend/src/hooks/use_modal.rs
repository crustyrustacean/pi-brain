// src/hooks/use_modal.rs

use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ModalType {
    CreateDocument,
    EditDocument(uuid::Uuid),
    ViewDocument(uuid::Uuid),
}

#[derive(Clone, PartialEq)]
pub struct ModalState {
    pub is_open: bool,
    pub modal_type: Option<ModalType>,
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            is_open: false,
            modal_type: None,
        }
    }
}

#[hook]
pub fn use_modal() -> UseModalHandle {
    let state = use_state(|| ModalState::default());
    
    let open_modal = {
        let state = state.clone();
        
        Callback::from(move |modal_type: ModalType| {
            state.set(ModalState {
                is_open: true,
                modal_type: Some(modal_type),
            });
        })
    };
    
    let close_modal = {
        let state = state.clone();
        
        Callback::from(move |_| {
            state.set(ModalState::default());
        })
    };
    
    UseModalHandle {
        state: (*state).clone(),
        open_modal,
        close_modal,
    }
}

#[derive(Clone)]
pub struct UseModalHandle {
    pub state: ModalState,
    pub open_modal: Callback<ModalType>,
    pub close_modal: Callback<()>,
}