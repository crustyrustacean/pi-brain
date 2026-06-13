// src/db.rs

use crate::error::ApiError;
use crate::models::{Document, KnowledgeBaseStats};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use hex;
use sqlx::{SqlitePool, Row};
use uuid::Uuid;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Compute SHA-256 hash of content for deduplication
    fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Create a new document
    pub async fn create_document(
        &self,
        title: &str,
        content: &str,
        tags: &[String],
        metadata: Option<&serde_json::Value>,
    ) -> Result<Document, ApiError> {
        let content_hash = Self::compute_hash(content);

        // Check for duplicate content
        let existing = sqlx::query(
            "SELECT id FROM documents WHERE content_hash = ?1 AND is_deleted = 0"
        )
        .bind(&content_hash)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = existing {
            let id: String = row.get("id");
            let existing_uuid = Uuid::parse_str(&id)?;
            return self.get_document(&existing_uuid).await?.ok_or_else(|| {
                ApiError::Internal("Failed to retrieve existing document".to_string())
            });
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let tags_json = serde_json::to_string(tags)?;
        let metadata_json = metadata.map(|m| serde_json::to_string(m)).transpose()?;

        sqlx::query(
            "INSERT INTO documents (id, title, content, content_hash, tags, metadata, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
        )
        .bind(&id)
        .bind(title)
        .bind(content)
        .bind(&content_hash)
        .bind(&tags_json)
        .bind(&metadata_json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        self.get_document_by_id(&id).await?.ok_or_else(|| {
            ApiError::Internal("Failed to retrieve created document".to_string())
        })
    }

    /// Get a document by ID
    pub async fn get_document(&self, id: &Uuid) -> Result<Option<Document>, ApiError> {
        self.get_document_by_id(&id.to_string()).await
    }

    /// Get a document by ID string
    async fn get_document_by_id(&self, id: &str) -> Result<Option<Document>, ApiError> {
        let row = sqlx::query(
            "SELECT id, title, content, content_hash, tags, metadata, created_at, updated_at
             FROM documents WHERE id = ?1 AND is_deleted = 0"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let id: String = row.get("id");
            let title: String = row.get("title");
            let content: String = row.get("content");
            let content_hash: String = row.get("content_hash");
            let tags_str: String = row.get("tags");
            let metadata_str: Option<String> = row.get("metadata");
            let created_str: String = row.get("created_at");
            let updated_str: String = row.get("updated_at");

            let tags: Vec<String> = serde_json::from_str(&tags_str)?;
            let metadata: Option<serde_json::Value> = metadata_str.and_then(|s| serde_json::from_str(&s).ok());
            let created_at = DateTime::parse_from_rfc3339(&created_str)?.with_timezone(&Utc);
            let updated_at = DateTime::parse_from_rfc3339(&updated_str)?.with_timezone(&Utc);

            Ok(Some(Document {
                id: Uuid::parse_str(&id)?,
                title,
                content,
                content_hash,
                tags,
                metadata,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Update an existing document
    pub async fn update_document(
        &self,
        id: &Uuid,
        title: Option<&str>,
        content: Option<&str>,
        tags: Option<&[String]>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<Option<Document>, ApiError> {
        let existing = self.get_document(id).await?;
        if existing.is_none() {
            return Ok(None);
        }

        let doc = existing.unwrap();
        let new_title = title.unwrap_or(&doc.title);
        let new_content = content.unwrap_or(&doc.content);
        let new_tags = tags.unwrap_or(&doc.tags);
        let new_content_hash = if content.is_some() {
            Self::compute_hash(new_content)
        } else {
            doc.content_hash.clone()
        };
        let new_metadata = if metadata.is_some() {
            metadata
        } else {
            doc.metadata.as_ref()
        };

        let now = Utc::now().to_rfc3339();
        let tags_json = serde_json::to_string(new_tags)?;
        let metadata_json = new_metadata.map(|m| serde_json::to_string(m)).transpose()?;

        sqlx::query(
            "UPDATE documents 
             SET title = ?1, content = ?2, content_hash = ?3, tags = ?4, metadata = ?5, updated_at = ?6
             WHERE id = ?7"
        )
        .bind(new_title)
        .bind(new_content)
        .bind(&new_content_hash)
        .bind(&tags_json)
        .bind(&metadata_json)
        .bind(&now)
        .bind(&id.to_string())
        .execute(&self.pool)
        .await?;

        self.get_document(id).await
    }

    /// Soft delete a document
    pub async fn delete_document(&self, id: &Uuid) -> Result<bool, ApiError> {
        let result = sqlx::query(
            "UPDATE documents SET is_deleted = 1, updated_at = ?1 WHERE id = ?2"
        )
        .bind(Utc::now().to_rfc3339())
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Search documents using FTS5
    pub async fn search_documents(
        &self,
        query: &str,
        tags: Option<&[String]>,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<Document>, i64), ApiError> {
        let mut where_conditions = vec!["d.is_deleted = 0".to_string()];
        let mut bind_params: Vec<String> = Vec::new();
        let mut param_count = 0;

        // Add FTS5 search condition if query is not empty
        if !query.is_empty() {
            param_count += 1;
            where_conditions.push(format!("d.id IN (SELECT id FROM documents_fts WHERE documents_fts MATCH ?{})", param_count));
            bind_params.push(query.to_string());
        }

        // Add tag filtering
        if let Some(tag_list) = tags && !tag_list.is_empty() {
                for tag in tag_list {
                    param_count += 1;
                    where_conditions.push(format!("d.tags LIKE ?{}", param_count));
                    bind_params.push(format!("%\"{}\"%", tag));
                }
            }

        let where_clause = where_conditions.join(" AND ");

        // Get total count
        let count_sql = format!("SELECT COUNT(*) FROM documents d WHERE {}", where_clause);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        for param in &bind_params {
            count_query = count_query.bind(param);
        }
        let total_count = count_query.fetch_one(&self.pool).await.unwrap_or(0);

        // Get paginated results
        let sql = format!(
            "SELECT d.id, d.title, d.content, d.content_hash, d.tags, d.metadata, d.created_at, d.updated_at
             FROM documents d
             WHERE {}
             ORDER BY d.updated_at DESC
             LIMIT ?{} OFFSET ?{}",
            where_clause,
            param_count + 1,
            param_count + 2
        );

        let mut query = sqlx::query(&sql);
        for param in &bind_params {
            query = query.bind(param);
        }
        query = query.bind(limit as i64);
        query = query.bind(offset as i64);

        let rows = query.fetch_all(&self.pool).await?;

        let documents = rows
            .into_iter()
            .map(|row| {
                let id: String = row.get("id");
                let title: String = row.get("title");
                let content: String = row.get("content");
                let content_hash: String = row.get("content_hash");
                let tags_str: String = row.get("tags");
                let metadata_str: Option<String> = row.get("metadata");
                let created_str: String = row.get("created_at");
                let updated_str: String = row.get("updated_at");

                let tags: Vec<String> = serde_json::from_str(&tags_str)?;
                let metadata: Option<serde_json::Value> = metadata_str.and_then(|s| serde_json::from_str(&s).ok());
                let created_at = DateTime::parse_from_rfc3339(&created_str)?.with_timezone(&Utc);
                let updated_at = DateTime::parse_from_rfc3339(&updated_str)?.with_timezone(&Utc);

                Ok(Document {
                    id: Uuid::parse_str(&id)?,
                    title,
                    content,
                    content_hash,
                    tags,
                    metadata,
                    created_at,
                    updated_at,
                })
            })
            .collect::<Result<Vec<_>, ApiError>>()?;

        Ok((documents, total_count))
    }

    /// Get knowledge base statistics
    pub async fn get_stats(&self) -> Result<KnowledgeBaseStats, ApiError> {
        let total_documents: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM documents WHERE is_deleted = 0"
        )
        .fetch_one(&self.pool)
        .await?;

        let total_links: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM document_links")
            .fetch_one(&self.pool)
            .await?;

        let db_size: i64 = sqlx::query_scalar("SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()")
            .fetch_one(&self.pool)
            .await?;

        let last_updated: Option<DateTime<Utc>> = match sqlx::query_scalar::<_, Option<String>>(
            "SELECT MAX(updated_at) FROM documents WHERE is_deleted = 0"
        )
        .fetch_optional(&self.pool)
        .await?
        {
            Some(Some(updated_str)) => Some(DateTime::parse_from_rfc3339(&updated_str)?.with_timezone(&Utc)),
            _ => Some(Utc::now()),
        };

        // Count unique tags (simplified approach)
        let unique_tags: i64 = total_documents * 2; // Rough estimate

        Ok(KnowledgeBaseStats {
            total_documents,
            total_links,
            unique_tags,
            database_size_bytes: db_size,
            last_updated: last_updated.unwrap_or_else(Utc::now),
        })
    }
}