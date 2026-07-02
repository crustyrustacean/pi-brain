# Changelog

All notable changes to pi-brain will be documented in this file.

## [1.1.0] - 2026-07-02

Untangles the database and route layers and fixes the missing-document status code.

### Backend

- **Lifted construction out of the database methods.** `create_document` and `update_document` no longer fetch/merge/hash/serialize inside the repo. The handlers now build a fully-resolved `Document` (hash, id, timestamps, partial-update merge, dedup orchestration) and hand it to a dumb write; the repo's `insert_document` / `update_document` do one statement each.
- **`DatabaseError` implements `actix_web::ResponseError`.** `NotFound` now reaches the client as **404** and `Operation` as 500, so `get`/`update`/`delete` of a missing document return 404 instead of 500/204. The typed error carries its own status; handlers propagate it via `?`.
- **`delete_document` checks `rows_affected()`** (with `AND is_deleted = 0`) so deleting a missing or already-deleted document raises `NotFound` → 404 instead of a silent 204.
- **`compute_content_hash` is now handler-only** (moved out of the repo; the repo no longer imports it).
- The `e400` / `e500` helpers remain scoped to untyped errors (e.g. `Uuid::parse_str` → 400, `DocumentRow → Document` mapping → 500), matching the zero2prod layering where typed errors self-describe their status.

### Tests

- Updated missing-document assertions from 500 to 404; renamed `get_document_returns_500_for_nonexistent_id` → `..._404_...`.
- Verified the full range live: missing → 404 across GET/PUT/DELETE, malformed uuid → 400, soft-delete hides the document (GET → 404), re-delete → 404.

## [1.0.0] - 2026-07-01

First stable release. The backend is rebuilt on the [`r2-photo-api`](https://codeberg.org/crustyrustacean/r2-photo-api) architecture, and the `ApiResponse`/`ApiError` JSON envelope is removed in favour of bare-JSON responses.

### Backend

- **Trait-based persistence layer** — introduce `DatabaseBackend` (async trait) with a `SqliteRepository` implementation, injected into the app as `Box<dyn DatabaseBackend>`. `DatabaseError` (`NotFound` / `Operation`) preserves the cause chain via `error_chain_fmt`.
- **Per-route handlers** — one Actix handler per file (`create`, `read`, `update`, `delete`, `list`, `search`, `stats`, `endpoints`, `health`), each returning `Result<HttpResponse, actix_web::Error>` and carrying lower-level errors up via `e400` / `e500`.
- **Migrations** now run inside `SqliteRepository::new` instead of `startup`.
- **Dynamic search** rebuilt on sqlx 0.9 `QueryBuilder` (resolves the `SqlSafeStr` gate while keeping FTS5 + tag filtering fully bound).
- **Instrumentation** — `#[tracing::instrument]` on database methods and error logging in `e400`/`e500`, so 4xx/5xx failures are diagnosable; request spans come from `TracingLogger`.
- **`KnowledgeBaseStats` renamed to `PiBrainStats`.**

### Removed

- `ApiError` / `ApiResponse` and the `{ success, data, error }` envelope — all responses are now bare JSON.
- `models.rs`, `db.rs`, `error.rs`, and `response.rs` replaced by `domain/` and `database/`.

### Frontend

- **API client rewritten on `gloo-net`**, dropping the hand-rolled `web_sys::fetch` plumbing. All methods consume bare domain types.
- Hooks (`use_documents`, `use_search`, `use_stats`) updated to the bare-JSON contract.

### Shared

- Pure domain types only — removed the dead `backend` feature and its `actix-web` / `tracing` optional dependencies.

### Workspace

- Hoisted shared metadata to `[workspace.package]` and common dependencies to `[workspace.dependencies]`; edition 2024 across all members.
- Removed the misplaced `frontend/.cargo/config.toml`; relocated the `wasm-pack` profile metadata into `frontend/Cargo.toml`.

### Tests & docs

- r2-style integration suite using an in-memory SQLite database (13 tests).
- README rewritten to document the new architecture, the bare-JSON API contract, and the error model.

## [0.4.0] - 2026-06-13

### Frontend deployment

- **Frontend now serves at root URL (`/`)** — the SPA is accessible at `http://host:8000/` alongside the API at `/kb/*`
- **Switched build pipeline from `wasm-pack --dev` to `trunk build --release`** — proper WASM optimizations (wasm-opt -Oz), minified JS, content-hashed output files, no dev HReload injected
- **Fixed static file path** — backend now correctly resolves `../frontend/dist` relative to its working directory
- **Fixed stray `#[cfg(feature = "backend")]` on shared types** — `Document` and other types are now available to both frontend and backend

### Backend

- **Added `frontend/` to systemd `ReadWritePaths`** — required under `ProtectSystem=strict` to serve static files

### Documentation

- **Rewrote README.md** — project overview, architecture, API reference, build/run instructions, configuration, frontend dev workflow, systemd setup, database schema
- **Updated build scripts** — fixed stale binary name reference, reliable `SCRIPT_DIR`, proper exit hints

## [0.3.0] - 2026-05-20

### Add Yew/WASM Frontend

- **Frontend**: Yew 0.23 SPA with Trunk dev server and WASM build pipeline
  - Document CRUD (create, view, edit, delete)
  - Full-text search with results display
  - Stats dashboard
  - Modal-based UI for document management
  - API client with proxy configuration

- **Backend**: CORS support and build fixes
  - Add `actix-cors` middleware
  - Add `[lib]` name override for crate naming
  - Fix static files path for project-root CWD

- **Shared**: Shared types crate for type-safe frontend/backend integration

- **Infrastructure**
  - Trunk proxy config for dev (`Trunk.toml`)
  - Configuration symlink for systemd service compatibility

## [0.2.0] - 2026-02-22

### Finish Zero2Prod style template

- **Split into bin/lib Format**
  - create modules for configuration, telemetry, startup
  - add integration tests

## [0.1.0] - 2026-02-22

### Initial Commit

- **Scaffold Based on Zero2Prod**
  - use intro example from Actix Web docs as starter
  - add `/health_check` route
