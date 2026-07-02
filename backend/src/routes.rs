// src/routes.rs

// module declarations
pub mod create;
pub mod delete;
pub mod endpoints;
pub mod health;
pub mod list;
pub mod read;
pub mod search;
pub mod stats;
pub mod update;

// re-exports
pub use create::*;
pub use delete::*;
pub use endpoints::*;
pub use health::*;
pub use list::*;
pub use read::*;
pub use search::*;
pub use stats::*;
pub use update::*;
