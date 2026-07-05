# pi-brain

A local knowledge base for AI coding agents. Persistent, searchable, accessible via REST API and a web UI.

## Overview

pi-brain gives AI assistants long-term memory. Documents are stored in a local SQLite database with full-text search, deduplicated content, and soft deletes. The backend exposes a REST API designed for programmatic access by tools and agents, while the frontend provides a human-friendly interface for browsing and managing stored knowledge.

## Architecture

```
pi-brain/
├── backend/              Actix Web REST API (Rust, edition 2024)
│   ├── src/
│   │   ├── bin/          Application entrypoint
│   │   ├── database/     `DatabaseBackend` trait + `SqliteRepository` (sqlx/SQLite)
│   │   ├── domain/       Domain types re-exported from `shared`
│   │   ├── routes/       One handler per endpoint (create, read, update, …)
│   │   └── configuration.rs / startup.rs / telemetry.rs / utils.rs
│   ├── configuration/    Layered YAML config (base + environment overlays)
│   └── migrations/       SQLite schema + FTS5 (run automatically on startup)
├── frontend/             Yew WASM SPA (Rust → WebAssembly), built with Trunk
├── shared/               Shared domain types crate (pure data types)
└── Cargo.toml            Workspace root (shared metadata + dependency versions)
```

- **Backend** — Actix Web 4 with SQLite (via sqlx) and FTS5 full-text search. Persistence sits behind a `DatabaseBackend` trait with a `SqliteRepository` implementation, injected into the app as a trait object (`Box<dyn DatabaseBackend>`). Structured JSON logging and layered YAML configuration.
- **Frontend** — Yew 0.23 compiled to WebAssembly via Trunk. Document CRUD, search, and a stats dashboard; talks to the API over `gloo-net`.
- **Shared** — Common domain types (`Document`, request/response DTOs, `PiBrainStats`) shared between frontend and backend for type-safe consistency. Pure data types with no backend framework dependencies.
- **Workspace** — Shared package metadata (`[workspace.package]`) and common dependency versions (`[workspace.dependencies]`) are declared once at the workspace root and inherited by each member crate.

## API

All data endpoints are under `/pb/`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health_check` | Liveness probe |
| POST | `/pb/documents` | Create a document |
| GET | `/pb/documents` | List documents (paginated) |
| GET | `/pb/documents/{id}` | Get a document by UUID |
| PUT | `/pb/documents/{id}` | Update a document |
| DELETE | `/pb/documents/{id}` | Soft-delete a document |
| POST | `/pb/search` | Full-text search (body) |
| GET | `/pb/search?q=` | Full-text search (query params) |
| GET | `/pb/stats` | Knowledge base statistics |
| GET | `/pb/endpoints` | API discovery (machine-readable) |

Responses are **bare JSON** — the entity (or DTO) is returned directly, with no
envelope wrapper. For example, `GET /pb/documents/{id}` returns the document:

```json
{
  "id": "9b1f7a2e-...-d4",
  "title": "Example",
  "content": "…",
  "content_hash": "sha256…",
  "tags": ["example"],
  "metadata": null,
  "created_at": "2026-07-01T12:00:00Z",
  "updated_at": "2026-07-01T12:00:00Z"
}
```

**Errors.** Route handlers return `Result<HttpResponse, actix_web::Error>` and carry
lower-level errors up via the `e400` / `e500` helpers in `utils.rs`. The database
layer's `DatabaseError` (`NotFound` / `Operation`) surfaces as HTTP 500, and a
malformed UUID in a path segment yields HTTP 400. (This mirrors the
[`r2-photo-api`](https://codeberg.org/crustyrustacean/r2-photo-api) error model.)

## Prerequisites

- Rust 1.85+ (edition 2024)
- [Trunk](https://trunkrs.dev/) (`cargo install --locked trunk`)
- wasm32 target (`rustup target add wasm32-unknown-unknown`)

## Build

```bash
# Build frontend (WASM) + backend (native)
./build.sh
```

Or individually:

```bash
# Frontend only — outputs to frontend/dist/
cd frontend && trunk build --release

# Backend only
cargo build --release
```

## Run

```bash
cargo run --release --bin pi-brain
```

The service listens on port 8000 by default:

| URL | Serves |
|-----|--------|
| `http://localhost:8000/` | Frontend SPA |
| `http://localhost:8000/pb/*` | REST API |

## Configuration

Configuration is layered from YAML files in `backend/configuration/`:

- `base.yaml` — defaults (all environments)
- `local.yaml` — local development overrides
- `production.yaml` — production overrides

Environment selection via `APP_ENVIRONMENT` (defaults to `local`). All settings can also be overridden with environment variables prefixed with `APP_` and separated with `__` (e.g. `APP_APPLICATION__PORT=9000`).

### Available settings

```yaml
application:
  port: 8000
  host: "0.0.0.0"
  base_url: "http://localhost:8000"
database:
  path: "/home/user/.pi/agent/data/pi-brain.db"
```

## Frontend Development

```bash
cd frontend
trunk serve          # Dev server with HMR at http://localhost:8080
```

Trunk proxies `/pb/*` requests to the backend at `http://localhost:8000` (see `Trunk.toml`).

## systemd Service

An example user service is provided for running pi-brain as a background daemon:

```ini
[Unit]
Description=Pi Brain API (Actix Web)
After=network.target

[Service]
Type=simple
WorkingDirectory=/path/to/pi-brain/backend
ExecStart=/path/to/pi-brain/target/release/pi-brain
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=APP_ENVIRONMENT=production

[Install]
WantedBy=default.target
```

```bash
systemctl --user daemon-reload
systemctl --user enable --now pi-brain
```

## Database

SQLite with FTS5. Migrations run automatically on startup. The schema includes:

- **`documents`** — UUID primary key, title, content, SHA-256 content hash (deduplication), tags (JSON), metadata (JSON), timestamps, soft-delete flag
- **`documents_fts`** — FTS5 virtual table with auto-sync triggers for full-text search
- **`document_links`** — Related document graph (planned)

Default database path: `~/.pi/agent/data/pi-brain.db`

## License

[MIT](License.txt)
