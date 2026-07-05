# Knowledge Base Frontend

A Yew-based frontend for the Knowledge Base application.

## Features

- **Document Management**: Create, read, update, and delete documents
- **Search**: Full-text search across all documents
- **Tagging**: Organize documents with tags
- **Statistics**: View database statistics
- **Responsive Design**: Clean, modern UI

## Architecture

The frontend follows a clean architecture with separation of concerns:

### Directory Structure

```
src/
├── api/           # API client and HTTP requests
├── hooks/         # Custom hooks for business logic
├── components/    # Reusable UI components
├── pages/         # Page-level components
└── main.rs        # Application entry point
```

### Key Design Principles

1. **Hooks-Based Approach**: All fetch logic and state management is separated from UI using custom hooks
2. **Type Safety**: Shared types with backend ensure compile-time safety
3. **Modular Components**: Reusable components that can be easily extended
4. **Clean State Management**: Each hook manages its own state and provides callbacks for actions

## Development

### Prerequisites

- Rust with wasm32 target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```

- wasm-pack:
  ```bash
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
  ```

### Building

```bash
# Development build
./build.sh

# Or manually:
wasm-pack build --dev --target web --out-dir dist
cp index.html dist/
```

### Running with Backend

The frontend is served by the Actix Web backend. Start the backend:

```bash
cd ..
cargo run --release --bin knowledge-base
```

Then access the application at `http://localhost:8000`

## Tech Stack

- **Yew 0.23**: Modern Rust framework for building web apps
- **gloo-net**: HTTP client for making API requests
- **serde**: Serialization/deserialization
- **shared crate**: Type-safe integration with backend

## Components

### Hooks

- `use_documents`: Document CRUD operations and state management
- `use_search`: Search functionality
- `use_stats`: Database statistics
- `use_modal`: Modal state management

### Components

- `DocumentList`: Display list of documents
- `SearchBar`: Search input and submission
- `StatsDisplay`: Statistics cards
- `DocumentModal`: Create/Edit/View document modal
- `LoadingSpinner`: Loading indicator
- `ErrorMessage`: Error display

## API Integration

The frontend communicates with the backend via REST API:

- `GET /pb/documents` - List documents
- `GET /pb/documents/{id}` - Get document
- `POST /pb/documents` - Create document
- `PUT /pb/documents/{id}` - Update document
- `DELETE /pb/documents/{id}` - Delete document
- `POST /pb/search` - Search documents
- `GET /pb/stats` - Get statistics

## Styling

The frontend uses inline CSS for simplicity. Styles are defined in `index.html` and can be easily customized or moved to a CSS framework.