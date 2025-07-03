# DSM Project Agent Configuration

## Build/Test/Lint Commands

**Rust Projects (Storage Node & DSM Core):**
- Build: `cargo build --release` (storage node), `cargo build --workspace` (dsm client)  
- Test: `cargo test --workspace` (all tests), `cargo test --package dsm_storage_node --lib storage::tests` (specific module)
- Single test: `cargo test test_name` or `cargo test -- --ignored test_name` (for ignored tests)
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Format: `cargo fmt --all` (check: `cargo fmt --all -- --check`)
- Security audit: `cargo audit`

**Web Frontend:**
- Build: `npm run build:webpack`, `npm run build:android` (with Android)
- Test: `npm run test`, `npm run ci` (full pipeline)
- Lint: `npm run lint`, `npm run lint:fix`
- Type check: `npm run type-check`
- Dev server: `npm start`

**Development cluster:** `cd dsm_storage_node && ./start_dev_cluster.sh`

## Architecture & Structure

**Core Projects:**
- `dsm_storage_node/`: Rust storage backend with HTTP API, consensus, and MPC genesis
- `dsm_client/decentralized_state_machine/`: Core DSM Rust library (workspace with `dsm` and `dsm_sdk` crates)
- `dsm_client/new_frontend/`: React TypeScript web frontend with WebView bridge
- `proto/`: Protocol buffer definitions for cross-platform communication

**APIs:** REST HTTP/JSON on ports 8080-8084 (dev), endpoints: `/api/v1/health`, `/api/v1/genesis/*`
**Database:** SQLite with epidemic gossip protocol for consensus
**Cryptography:** Post-quantum (ML-KEM, SPHINCS+) + classical (AES-256-GCM, SHA-3)

## Code Style & Conventions

**Rust:**
- Format: 4 spaces, max_width=100, Unix newlines (see `rustfmt.toml`)
- Error handling: Use `thiserror` for errors, `anyhow` for utilities, `Result<T, Error>` patterns
- Imports: Preserve order, no auto-reordering, grouped by std/external/local
- Clippy: All warnings as errors, allows specific lints in Cargo.toml `[lints.clippy]`
- Naming: snake_case for functions/variables, PascalCase for types, SCREAMING_SNAKE_CASE for constants

**TypeScript/React:**
- ESLint with React, security, and TypeScript rules
- Styled-components for CSS-in-JS
- Async/await patterns, no QR codes (incompatible with SPHINCS+ signature size)
