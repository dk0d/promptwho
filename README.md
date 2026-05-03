<div align="center">
  <img src='./assets/promptwho-logo.png' width='50%' alt='promptwho-logo'/>
</div>

# `promptwho`

Rust-first attribution and observability workspace for local AI-assisted development.

`promptwho` ingests editor, tool, and git activity into a local server so those events can be stored, queried, and attributed back to projects, sessions, and code changes.

## Getting started

Start the local server:

```bash
cargo run -p promptwho-cli -- serve
```

In another terminal, watch a local git repository for new commits:

```bash
cargo run -p promptwho-cli -- watch git --git-dir .
```

Inspect the effective configuration:

```bash
cargo run -p promptwho-cli -- config
```

Default local endpoints:

- server: `http://127.0.0.1:8765`
- health: `http://127.0.0.1:8765/readyz`
- docs: `http://127.0.0.1:8765/docs`

## How it works

The current local architecture is intentionally simple:

1. An event source emits promptwho-compatible events.
2. The local server accepts event batches at `/v1/events`.
3. The storage backend persists projects, sessions, messages, and events.
4. Query and dashboard endpoints expose the stored data for inspection.

Today there are two event sources in the repo:

- `plugins/opencode` captures Opencode plugin events and publishes MsgPack batches over HTTP
- `pwho watch git` emits `git.commit` events for new commits in a local repository

## Workspace crates

- `crates/promptwho-protocol` shared event and server API types
- `crates/promptwho-storage` domain storage traits, models, and queries
- `crates/promptwho-storage-surreal` first SurrealDB storage backend scaffold
- `crates/promptwho-core` ingest and attribution service layer
- `crates/promptwho-server` local axum server with OpenAPI/Scalar docs
- `crates/promptwho-cli` CLI entrypoint for `serve` and future queries
- `crates/promptwho-watcher` local watcher and event emitter support
- `crates/promptwho-bootstrap` bootstrap utilities and scaffolding

## Repository layout

- `crates/` Rust workspace crates
- `plugins/opencode/` thin Bun/TypeScript plugin for Opencode
- `dashboard/` local dashboard UI scaffold for browsing promptwho data
- `assets/` project branding and images

## Transport

- `plugins/opencode` thin Bun/TypeScript plugin scaffold
- plugin sends MsgPack event batches to the local HTTP server
- server ingests events and will expose query APIs over time
- local storage defaults to a file-backed SurrealDB store at `surrealkv://.promptwho/db`

## Configuration

Configuration is loaded from, in order of precedence:

1. built-in defaults
2. `./promptwho.toml` if present
3. XDG config at `~/.config/promptwho/promptwho.toml` when available
4. environment variables prefixed with `PROMPTWHO_`
5. an explicit file passed via `--config` or `PROMPTWHO_CONFIG`

Default values:

- server host: `127.0.0.1`
- server port: `8765`
- storage backend: `surreal`
- storage endpoint: `surrealkv://.promptwho/db`
- namespace: `promptwho`
- database: `promptwho`

Example `promptwho.toml`:

```toml
[server]
host = "127.0.0.1"
port = 8765

[storage]
backend = "surreal"
endpoint = "surrealkv://.promptwho/db"
namespace = "promptwho"
database = "promptwho"
```

## API overview

Current server routes include:

- `GET /readyz` health check
- `POST /v1/events` ingest event batches
- `GET /v1/projects` list known projects
- `GET /v1/sessions` list sessions
- `GET /v1/sessions/{session_id}/messages` list messages for a session
- `GET /v1/events/query` query stored events
- `GET /v1/search` text search across stored promptwho data
- `GET /docs` interactive API docs

## Current status

- protocol and trait boundaries are defined
- configuration is centralized in `crates/promptwho-core/src/config.rs`
- storage trait remains backend-agnostic
- SurrealDB is the first schemaless backend for rapid iteration
- local server replaces the previous daemon/IPC direction
- OpenAPI docs are intended to be served from Scalar at `/docs`

Implemented today:

- local server startup via `pwho serve`
- local git commit watching via `pwho watch git`
- MsgPack ingest pipeline for event batches
- dashboard-oriented read endpoints for projects, sessions, messages, events, and search

Still scaffolded or incomplete:

- `pwho doctor`
- `pwho query`
- broader attribution and query workflows beyond the current dashboard endpoints

## Git watcher

The CLI can watch a local git repository for new commits and publish `git.commit` events to `promptwho-server`.

```bash
pwho serve
pwho watch git --git-dir .
```

Notes:
- commit events include branch, HEAD commit, author, commit timestamp, title, and body
- the watcher stores its last-seen HEAD in `.promptwho/git-watcher.json` to avoid replaying old commits on restart

## Development

Common commands:

```bash
cargo test --workspace
cargo run -p promptwho-cli -- config
cargo run -p promptwho-cli -- serve
```

For the Opencode plugin:

```bash
cd plugins/opencode
bun run check
bun run build
```

## Limitations

- the repository is still in an early scaffold stage
- the dashboard exists, but the main integration story is still centered on the local Rust server and event ingestion pipeline
- storage currently defaults to a local file-backed SurrealKV setup for fast iteration rather than production deployment
