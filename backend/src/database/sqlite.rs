// src/database/sqlite.rs

use crate::configuration::DatabaseSettings;
use crate::database::{DatabaseBackend, DatabaseError, DocumentRow};
use crate::domain::{Document, PiBrainStats};
use anyhow::Context;
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{QueryBuilder, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;

/// Columns selected for every document read. Kept in sync with `DocumentRow`.
const SELECT_COLUMNS: &str = "id, title, content, content_hash, tags, metadata, created_at, updated_at";

/// SHA-256 content hash, used for deduplication.
fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

#[derive(Debug, Clone)]
pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub async fn new(db_configuration: &DatabaseSettings) -> Result<Self, anyhow::Error> {
        let db_path = format!("sqlite:{}", db_configuration.path);
        let mut pool_opts = SqlitePoolOptions::new();
        if let Some(max) = db_configuration.max_connections {
            pool_opts = pool_opts.max_connections(max);
        }
        let options = SqliteConnectOptions::from_str(&db_path)
            .context("Failed to parse the database connection string.")?
            .create_if_missing(true);
        let pool = pool_opts
            .connect_with(options)
            .await
            .context("Failed to connect to SQLite.")?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("Failed to run the database migrations.")?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl DatabaseBackend for SqliteRepository {
    #[tracing::instrument(skip(self, content, tags, metadata))]
    async fn create_document(
        &self,
        title: &str,
        content: &str,
        tags: &[String],
        metadata: Option<&serde_json::Value>,
    ) -> Result<DocumentRow, DatabaseError> {
        let content_hash = compute_content_hash(content);

        // Deduplicate by content hash — return the existing document if present.
        let existing: Option<(String,)> =
            sqlx::query_as("SELECT id FROM documents WHERE content_hash = ? AND is_deleted = 0")
                .bind(&content_hash)
                .fetch_optional(&self.pool)
                .await
                .context("Failed to check for duplicate content.")?;
        if let Some((id_str,)) = existing {
            let id =
                Uuid::parse_str(&id_str).context("Failed to parse existing document id.")?;
            return self.get_document(id).await;
        }

        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        let tags_json = serde_json::to_string(tags).context("Failed to serialize tags.")?;
        let metadata_json = metadata
            .map(serde_json::to_string)
            .transpose()
            .context("Failed to serialize metadata.")?;

        sqlx::query(
            "INSERT INTO documents (id, title, content, content_hash, tags, metadata, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(title)
        .bind(content)
        .bind(&content_hash)
        .bind(&tags_json)
        .bind(metadata_json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .context("Failed to create the document.")?;

        self.get_document(id).await
    }

    #[tracing::instrument(skip(self))]
    async fn get_document(&self, id: Uuid) -> Result<Document, DatabaseError> {
        let row: Option<DocumentRow> = sqlx::query_as(
            "SELECT id, title, content, content_hash, tags, metadata, created_at, updated_at
             FROM documents WHERE id = ? AND is_deleted = 0",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch the document.")?;
        let row = row.ok_or_else(|| DatabaseError::NotFound(id.to_string()))?;

        Ok(row.try_into().context("Failed to map the document.")?)
    }

    #[tracing::instrument(skip(self, content, tags, metadata))]
    async fn update_document(
        &self,
        id: Uuid,
        title: Option<&str>,
        content: Option<&str>,
        tags: Option<&[String]>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<Document, DatabaseError> {
        // Partial update: fall back to the existing values for absent fields.
        let existing = self.get_document(id).await?;

        let new_title = title.unwrap_or(&existing.title);
        let new_content = content.unwrap_or(&existing.content);
        let new_tags = tags.unwrap_or(&existing.tags);
        let new_content_hash = if content.is_some() {
            compute_content_hash(new_content)
        } else {
            existing.content_hash.clone()
        };
        let new_metadata = metadata.or(existing.metadata.as_ref());

        let now = Utc::now().to_rfc3339();
        let tags_json = serde_json::to_string(new_tags).context("Failed to serialize tags.")?;
        let metadata_json = new_metadata
            .map(serde_json::to_string)
            .transpose()
            .context("Failed to serialize metadata.")?;

        sqlx::query(
            "UPDATE documents
             SET title = ?, content = ?, content_hash = ?, tags = ?, metadata = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(new_title)
        .bind(new_content)
        .bind(&new_content_hash)
        .bind(&tags_json)
        .bind(metadata_json)
        .bind(&now)
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .context("Failed to update the document.")?;

        self.get_document(id).await
    }

    #[tracing::instrument(skip(self))]
    async fn delete_document(&self, id: Uuid) -> Result<(), DatabaseError> {
        sqlx::query("UPDATE documents SET is_deleted = 1, updated_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .context("Failed to delete the document.")?;

        Ok(())
    }

    #[tracing::instrument(skip(self, tags))]
    async fn search_documents(
        &self,
        query: &str,
        tags: Option<&[String]>,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<Document>, i64), DatabaseError> {
        let tag_list = tags.filter(|t| !t.is_empty());

        // --- total match count (for pagination metadata) ---
        let mut count: QueryBuilder<sqlx::Sqlite> =
            QueryBuilder::new("SELECT COUNT(*) FROM documents WHERE is_deleted = 0");
        if !query.is_empty() {
            count.push(" AND id IN (SELECT id FROM documents_fts WHERE documents_fts MATCH ");
            count.push_bind(query.to_string());
            count.push(")");
        }
        if let Some(tags) = tag_list {
            for tag in tags {
                count.push(" AND tags LIKE ");
                count.push_bind(format!("%\"{tag}\"%"));
            }
        }
        let total: i64 = count
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .context("Failed to count documents.")?;

        // --- paginated rows ---
        let mut rows: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new("SELECT ");
        rows.push(SELECT_COLUMNS);
        rows.push(" FROM documents WHERE is_deleted = 0");
        if !query.is_empty() {
            rows.push(" AND id IN (SELECT id FROM documents_fts WHERE documents_fts MATCH ");
            rows.push_bind(query.to_string());
            rows.push(")");
        }
        if let Some(tags) = tag_list {
            for tag in tags {
                rows.push(" AND tags LIKE ");
                rows.push_bind(format!("%\"{tag}\"%"));
            }
        }
        rows.push(" ORDER BY updated_at DESC LIMIT ");
        rows.push_bind(limit as i64);
        rows.push(" OFFSET ");
        rows.push_bind(offset as i64);

        let rows = rows
            .build_query_as::<DocumentRow>()
            .fetch_all(&self.pool)
            .await
            .context("Failed to search documents.")?;

        let documents = rows
            .into_iter()
            .map(Document::try_from)
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to map documents.")?;

        Ok((documents, total))
    }

    #[tracing::instrument(skip(self))]
    async fn get_stats(&self) -> Result<PiBrainStats, DatabaseError> {
        let total_documents: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM documents WHERE is_deleted = 0")
                .fetch_one(&self.pool)
                .await
                .context("Failed to count documents.")?;

        let total_links: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM document_links")
            .fetch_one(&self.pool)
            .await
            .context("Failed to count document links.")?;

        let database_size_bytes: i64 = sqlx::query_scalar(
            "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to read the database size.")?;

        let last_updated_raw: Option<Option<String>> =
            sqlx::query_scalar("SELECT MAX(updated_at) FROM documents WHERE is_deleted = 0")
                .fetch_optional(&self.pool)
                .await
                .context("Failed to read the last updated timestamp.")?;
        let last_updated = last_updated_raw
            .flatten()
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)))
            .unwrap_or_else(Utc::now);

        // TODO: compute the real unique-tag count instead of this rough estimate.
        let unique_tags = total_documents * 2;

        Ok(PiBrainStats {
            total_documents,
            total_links,
            unique_tags,
            database_size_bytes,
            last_updated,
        })
    }
}
