# `promptwho`

First-pass workspace scaffold for a Rust-first attribution tool with a thin Opencode TypeScript plugin and a local server-client architecture.

## Workspace crates

- `crates/promptwho-protocol` shared event and server API types
- `crates/promptwho-storage` domain storage traits, models, and queries
- `crates/promptwho-storage-surreal` first SurrealDB storage backend scaffold
- `crates/promptwho-core` ingest and attribution service layer
- `crates/promptwho-server` local axum server with OpenAPI/Scalar docs
- `crates/promptwho-cli` CLI entrypoint for `serve` and future queries

## Transport

- `plugins/opencode` thin Bun/TypeScript plugin scaffold
- plugin sends MsgPack event batches to the local HTTP server
- server ingests events and will expose query APIs over time
- local storage defaults to a file-backed SurrealDB store at `surrealkv://.promptwho/db`

## Current status

- protocol and trait boundaries are defined
- configuration is centralized in `crates/promptwho-core/src/config.rs`
- storage trait remains backend-agnostic
- SurrealDB is the first schemaless backend for rapid iteration
- local server replaces the previous daemon/IPC direction
- OpenAPI docs are intended to be served from Scalar at `/docs`
