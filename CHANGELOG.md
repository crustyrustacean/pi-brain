# Changelog

All notable changes to pi-brain will be documented in this file.

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
