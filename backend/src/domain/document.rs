// src/domain/document.rs

// The canonical document types live in the `pi-brain-shared` crate so that the
// Yew frontend and the Actix backend remain type-aligned. Re-export them here as
// the backend's domain layer, mirroring r2-photo-api's `domain/photo.rs`.
pub use pi_brain_shared::*;
