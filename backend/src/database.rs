// src/database.rs

use crate::domain::{Document, PiBrainStats};
use crate::utils::error_chain_fmt;
use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
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
    ) -> Result<DocumentRow, DatabaseError>;

    async fn get_document(&self, id: Uuid) -> Result<DocumentRow, DatabaseError>;

    async fn update_document(
        &self,
        id: Uuid,
        title: Option<&str>,
        content: Option<&str>,
        tags: Option<&[String]>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<DocumentRow, DatabaseError>;

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


#[derive(Debug, FromRow, Serialize)]
struct DocumentRow {
    id: String,
    title: String,
    content: String,
    content_hash: String,
    tags: String,
    metadata: Option<String>,
    created_at: String,
    updated_at: String,
}

impl TryFrom<DocumentRow> for Document {
    type Error = anyhow::Error;

    fn try_from(row: DocumentRow) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(&row.id).context("Failed to parse document id.")?;
        let tags: Vec<String> =
            serde_json::from_str(&row.tags).context("Failed to parse document tags.")?;
        let metadata = row
            .metadata
            .filter(|s| !s.is_empty())
            .map(|s| serde_json::from_str::<serde_json::Value>(&s))
            .transpose()
            .context("Failed to parse document metadata.")?;
        let created_at = DateTime::parse_from_rfc3339(&row.created_at)
            .context("Failed to parse created_at.")?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&row.updated_at)
            .context("Failed to parse updated_at.")?
            .with_timezone(&Utc);

        Ok(Document {
            id,
            title: row.title,
            content: row.content,
            content_hash: row.content_hash,
            tags,
            metadata,
            created_at,
            updated_at,
        })
    }
}
