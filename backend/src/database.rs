// src/database.rs

use crate::domain::{Document, PiBrainStats};
use crate::utils::error_chain_fmt;
use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

pub mod sqlite;

pub use sqlite::SqliteRepository;

#[derive(Error)]
pub enum DatabaseError {
    #[error("record not found: {0}")]
    NotFound(String),

    #[error(transparent)]
    Operation(#[from] anyhow::Error),
}

impl std::fmt::Debug for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[async_trait]
pub trait DatabaseBackend: Send + Sync {
    async fn create_document(
        &self,
        title: &str,
        content: &str,
        tags: &[String],
        metadata: Option<&serde_json::Value>,
    ) -> Result<Document, DatabaseError>;

    async fn get_document(&self, id: Uuid) -> Result<Document, DatabaseError>;

    async fn update_document(
        &self,
        id: Uuid,
        title: Option<&str>,
        content: Option<&str>,
        tags: Option<&[String]>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<Document, DatabaseError>;

    async fn delete_document(&self, id: Uuid) -> Result<(), DatabaseError>;

    async fn search_documents(
        &self,
        query: &str,
        tags: Option<&[String]>,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<Document>, i64), DatabaseError>;

    async fn get_stats(&self) -> Result<PiBrainStats, DatabaseError>;
}
