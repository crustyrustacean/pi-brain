// src/routes.rs

// module declarations
pub mod documents;
pub mod endpoints;
pub mod health_check;
pub mod search;
pub mod stats;

// re-exports
pub use documents::*;
pub use endpoints::*;
pub use health_check::*;
pub use search::*;
pub use stats::*;