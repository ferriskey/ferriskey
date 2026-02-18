# Local Development (with `just`)

This repo uses a `justfile` (<https://github.com/casey/just>) to provide a consistent set of local development commands.

## Prerequisites

- `just` installed and on your PATH
- Docker (with `docker compose` v2)
- Rust toolchain (for API development)
- Node.js + pnpm (for frontend development)

Some `just` recipes will offer to install missing tools (for example Node/pnpm/Docker) and will fail if you choose not to install them.

## Discover Commands

List all available recipes:

```bash
just --list
```

Get help (same as `just --list` in this repo):

```bash
just
just help
```

## Typical Local Workflow

### 1) First-time setup (database + migrations)

This will:

- create `api/.env` from `api/env.example` if missing
- start the Postgres container
- wait for Postgres to be ready
- prompt you to run migrations

```bash
just dev-setup
```

If you skipped migrations (or want to re-run them later):

```bash
just migrate
```

### 2) Run the API (Rust, auto-reload)

Runs the API using `cargo watch`:

```bash
just dev
```

Notes:

- `api/.env` is created automatically (from `api/env.example`) if missing.
- The API reads config from `api/.env` (via dotenv).

### 3) Run the frontend (Vite dev server)

```bash
just web
```

Notes:

- `front/.env` is created automatically (from `front/.env.example`) if missing.
- The dev server is started with `--host 0.0.0.0 --port 5555`.

## Docker Compose "Full Stack" (Build Profile)

Bring up the full stack with Docker Compose (build + run containers):

```bash
just dev-test
```

Bring up the same stack with HTTPS on localhost:

```bash
just dev-test-ssl
```

URLs:

- Frontend: `https://localhost:5443`
- API (proxied through frontend): `https://localhost:5443/api`

Tear it down (including volumes):

```bash
just dev-test-down
```

## Database Reset / Cleanup

Stop and remove Docker Compose containers and delete their volumes (destructive for local DB data):

```bash
just db-down
```

## Environment Variables

- API env file: `api/.env` (copied from `api/env.example`)
- Frontend env file: `front/.env` (copied from `front/.env.example`)

`just migrate` uses `DATABASE_URL` if it is already set in your environment; otherwise it builds a `DATABASE_URL` from values in `api/.env`.
