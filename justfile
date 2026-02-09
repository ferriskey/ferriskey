set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# FerrisKey developer task runner
#
# Usage:
#   just                # same as: just help
#   just help           # list available recipes
#   just <recipe> ...   # run a recipe
#
# Common local workflows:
#   1) Start DB + run migrations (first time):
#        just dev-setup
#   2) Run API with auto-reload (Rust):
#        just dev
#   3) Run frontend dev server (Node/pnpm):
#        just web
#
# Notes:
# - Some recipes may prompt to install missing tools (Docker, Node, pnpm, etc).
# - Environment files are created on-demand:
#     api/.env  from api/env.example
#     front/.env from front/.env.example

# docker compose wrapper (always uses repo docker-compose.yaml)
compose := "docker compose -f docker-compose.yaml"

# Default Postgres data volume used by docker-compose.yaml. Override via:
#   PGDATA_VOLUME=custom_name just db-down
pgdata_volume_default := "ferriskey_pgdata"

default: help

# List all recipes defined in this file.
help:
  @just --list

# Internal helpers.
# Naming convention: recipes starting with '_' are not meant to be called directly.
_ensure-docker:
  @if command -v docker >/dev/null 2>&1; then \
    exit 0; \
  fi; \
  echo "Docker not found."; \
  read -r -p "Install Docker now using get.docker.com? [y/N] " ans; \
  case "${ans:-}" in \
    y|Y|yes|YES) ;; \
    *) echo "Skipping Docker install. Install Docker, then re-run." >&2; exit 1;; \
  esac; \
  if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then \
    echo "Need 'curl' or 'wget' to install Docker." >&2; \
    exit 1; \
  fi; \
  install_cmd=''; \
  if command -v curl >/dev/null 2>&1; then install_cmd='curl -fsSL https://get.docker.com | sh'; \
  else install_cmd='wget -qO- https://get.docker.com | sh'; fi; \
  if command -v sudo >/dev/null 2>&1; then sudo sh -lc "$install_cmd"; else sh -lc "$install_cmd"; fi; \
  echo "Docker install finished. If this is Linux, you may need to log out/in for group permissions."

_ensure-docker-running: _ensure-docker
  @if ! docker info >/dev/null 2>&1; then \
    echo "Docker daemon is not running (or you don't have permission)." >&2; \
    echo "Start Docker Desktop (macOS/Windows) or start the daemon (Linux), then re-run." >&2; \
    exit 1; \
  fi
  @if ! docker compose version >/dev/null 2>&1; then \
    echo "'docker compose' is not available." >&2; \
    echo "Install Docker Compose v2, then re-run." >&2; \
    exit 1; \
  fi

_ensure-node:
  @if command -v node >/dev/null 2>&1; then \
    exit 0; \
  fi; \
  echo "Node.js not found."; \
  read -r -p "Install latest Node LTS via nvm? [y/N] " ans; \
  case "${ans:-}" in \
    y|Y|yes|YES) ;; \
    *) echo "Skipping Node install." >&2; exit 1;; \
  esac; \
  if ! command -v curl >/dev/null 2>&1; then \
    echo "Need 'curl' to install nvm." >&2; \
    exit 1; \
  fi; \
  export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"; \
  if [ ! -s "$NVM_DIR/nvm.sh" ]; then \
    NVM_VERSION="$(curl -fsSL https://api.github.com/repos/nvm-sh/nvm/releases/latest | sed -n 's/.*\"tag_name\"[[:space:]]*:[[:space:]]*\"\\([^\\\"]*\\)\".*/\\1/p' | head -n1 || true)"; \
    if [ -z "${NVM_VERSION:-}" ]; then NVM_VERSION="v0.40.04"; fi; \
    curl -fsSL "https://raw.githubusercontent.com/nvm-sh/nvm/${NVM_VERSION}/install.sh" | bash; \
  fi; \
  . "$NVM_DIR/nvm.sh"; \
  nvm install --lts; \
  nvm use --lts

_ensure-pnpm: _ensure-node
  @if command -v pnpm >/dev/null 2>&1; then \
    exit 0; \
  fi; \
  echo "pnpm not found."; \
  read -r -p "Install latest pnpm via corepack? [y/N] " ans; \
  case "${ans:-}" in \
    y|Y|yes|YES) ;; \
    *) echo "Skipping pnpm install." >&2; exit 1;; \
  esac; \
  corepack enable; \
  corepack prepare pnpm@latest --activate

_ensure-cargo-watch:
  @if ! command -v cargo >/dev/null 2>&1; then \
    echo "Rust toolchain not found (missing 'cargo'). Install rustup, then re-run." >&2; \
    exit 1; \
  fi; \
  if command -v cargo-watch >/dev/null 2>&1; then \
    exit 0; \
  fi; \
  echo "cargo-watch not found."; \
  read -r -p "Install cargo-watch (latest) now? [y/N] " ans; \
  case "${ans:-}" in \
    y|Y|yes|YES) ;; \
    *) echo "Skipping cargo-watch install." >&2; exit 1;; \
  esac; \
  cargo install cargo-watch

_ensure-sqlx-cli:
  @if command -v sqlx >/dev/null 2>&1; then \
    exit 0; \
  fi; \
  if ! command -v cargo >/dev/null 2>&1; then \
    echo "Missing 'sqlx' (sqlx-cli) and Rust toolchain (missing 'cargo'). Install rustup, then re-run." >&2; \
    exit 1; \
  fi; \
  echo "sqlx-cli not found."; \
  read -r -p "Install sqlx-cli (postgres) now? [y/N] " ans; \
  case "${ans:-}" in \
    y|Y|yes|YES) ;; \
    *) echo "Skipping sqlx-cli install." >&2; exit 1;; \
  esac; \
  cargo install sqlx-cli --no-default-features --features postgres

_wait-db: _ensure-docker-running
  @# Wait for the Postgres container to accept connections.
  @if [ ! -f api/.env ]; then cp api/env.example api/.env; fi
  @set -a; . api/.env; set +a; \
    echo "Waiting for Postgres to accept connections..."; \
    for i in {1..60}; do \
      if {{compose}} exec -T db pg_isready -U "${DATABASE_USER:-ferriskey}" -d "${DATABASE_NAME:-ferriskey}" >/dev/null 2>&1; then \
        exit 0; \
      fi; \
      sleep 1; \
    done; \
    echo "Postgres did not become ready in time." >&2; \
    exit 1

dev-setup: _ensure-docker-running
  @# Bootstrap local development prerequisites:
  @# - Create api/.env if missing
  @# - Start the Postgres container
  @# - Optionally run SQL migrations
  @if [ ! -f api/.env ]; then cp api/env.example api/.env; fi
  @POSTGRES_IMAGE=postgres:18.1 POSTGRES_DATA_PATH=/var/lib/postgresql {{compose}} up -d db
  @just _wait-db
  @read -r -p "Run DB migrations now? [Y/n] " ans; \
    case "${ans:-Y}" in \
      n|N|no|NO) echo "Skipping migrations. You can run: just migrate";; \
      *) just migrate;; \
    esac

migrate: _ensure-sqlx-cli
  @# Apply SQL migrations from core/migrations.
  @# Uses DATABASE_URL if set; otherwise constructs it from api/.env values.
  @# NOTE: When constructing DATABASE_URL, DATABASE_USER and DATABASE_PASSWORD are percent-encoded first.
  @if [ -n "${DATABASE_URL:-}" ]; then \
    sqlx migrate run --source core/migrations; \
    exit 0; \
  fi; \
  if [ ! -f api/.env ]; then cp api/env.example api/.env; fi; \
  set -a; . api/.env; set +a; \
  : "${DATABASE_HOST:=localhost}"; \
  : "${DATABASE_PORT:=5432}"; \
  : "${DATABASE_NAME:=ferriskey}"; \
  : "${DATABASE_USER:=ferriskey}"; \
  : "${DATABASE_PASSWORD:=ferriskey}"; \
  if command -v python3 >/dev/null 2>&1; then \
    DATABASE_USER_ENC="$(python3 -c 'import sys, urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "${DATABASE_USER}")"; \
    DATABASE_PASSWORD_ENC="$(python3 -c 'import sys, urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "${DATABASE_PASSWORD}")"; \
  elif command -v python >/dev/null 2>&1; then \
    DATABASE_USER_ENC="$(python -c 'import sys\ntry:\n  from urllib.parse import quote\nexcept ImportError:\n  from urllib import quote\nprint(quote(sys.argv[1], safe=""))' "${DATABASE_USER}")"; \
    DATABASE_PASSWORD_ENC="$(python -c 'import sys\ntry:\n  from urllib.parse import quote\nexcept ImportError:\n  from urllib import quote\nprint(quote(sys.argv[1], safe=""))' "${DATABASE_PASSWORD}")"; \
  else \
    echo "Missing python3/python to percent-encode DATABASE_USER/DATABASE_PASSWORD when building DATABASE_URL." >&2; \
    echo "Workaround: set DATABASE_URL directly (recommended), or ensure DATABASE_USER/DATABASE_PASSWORD are already percent-encoded." >&2; \
    exit 1; \
  fi; \
  export DATABASE_URL="postgres://${DATABASE_USER_ENC}:${DATABASE_PASSWORD_ENC}@${DATABASE_HOST}:${DATABASE_PORT}/${DATABASE_NAME}"; \
  sqlx migrate run --source core/migrations

dev: _ensure-cargo-watch
  @# Run the Rust API locally with auto-reload on file changes.
  @# Ensure api/.env exists (app loads it via dotenv)
  @if [ ! -f api/.env ]; then cp api/env.example api/.env; fi
  @cd api && cargo watch -x "run --bin ferriskey-api"

dev-test: _ensure-docker-running
  @# Bring up the full stack using docker compose "build" profile (build + run containers).
  @# "Full" build profile run (build + run containers)
  @{{compose}} --profile build up -d --build

db-down: _ensure-docker-running
  @# Stop and remove only Postgres + its data volume.
  @# This is destructive for local data (drops your local DB state).
  @# Remove only the db container and its data volume (default: ferriskey_pgdata).
  @{{compose}} down -v db || true

dev-test-down: _ensure-docker-running
  @# Tear down docker compose build profile containers and volumes.
  @{{compose}} --profile build down -v

web: _ensure-pnpm
  @# Run the frontend dev server (Vite) locally.
  @if [ ! -f front/.env ]; then cp front/.env.example front/.env; fi
  @cd front && pnpm install && pnpm run dev --host 0.0.0.0 --port 5555
