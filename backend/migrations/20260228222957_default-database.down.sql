-- Drop triggers and FTS table
DROP TRIGGER IF EXISTS documents_ai;
DROP TRIGGER IF EXISTS documents_ad;
DROP TRIGGER IF EXISTS documents_au;
DROP TABLE IF EXISTS documents_fts;

-- Drop indexes
DROP INDEX IF EXISTS idx_document_links_target;
DROP INDEX IF EXISTS idx_document_links_source;
DROP INDEX IF EXISTS idx_documents_updated_at;
DROP INDEX IF EXISTS idx_documents_created_at;
DROP INDEX IF EXISTS idx_documents_content_hash;

-- Drop tables
DROP TABLE IF EXISTS document_links;
DROP TABLE IF EXISTS documents;
