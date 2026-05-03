# `promptwho` dashboard

SvelteKit dashboard for browsing locally ingested promptwho data.

The dashboard is a thin UI over the local `promptwho-server` API. It reads projects, sessions, messages, events, and search results from the server and renders them for local inspection.

## Prerequisites

- Bun installed
- the promptwho server running locally, usually at `http://127.0.0.1:8765`

Start the server from the workspace root:

```bash
cargo run -p promptwho-cli -- serve
```

## Development

From `dashboard/`:

```bash
bun install
bun run dev
```

Useful commands:

```bash
bun run check
bun run build
bun run preview
```

## Configuration

The dashboard reads the backend base URL from `PROMPTWHO_BASE_URL`.

Default:

```bash
PROMPTWHO_BASE_URL=http://127.0.0.1:8765
```

Example:

```bash
PROMPTWHO_BASE_URL=http://127.0.0.1:8765 bun run dev
```

If `PROMPTWHO_BASE_URL` is not set, the dashboard defaults to `http://127.0.0.1:8765`.

## What it loads

The dashboard currently reads from these server routes:

- `GET /v1/projects`
- `GET /v1/sessions`
- `GET /v1/sessions/{session_id}/messages`
- `GET /v1/events/query`
- `GET /v1/search`

If the server is unavailable or returns an error, the dashboard surfaces that failure in the UI rather than writing directly to storage itself.

## Notes

- this package is local-development focused
- the main backend contract lives in the Rust server, not in the dashboard
- UI behavior depends on promptwho data already being ingested by the server
