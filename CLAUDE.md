# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

FerrisKey is a modern, open-source Identity & Access Management (IAM) system built in Rust with a hexagonal architecture. It consists of:
- **core** - Business logic and domain entities (hexagonal architecture)
- **api** - HTTP API layer built with Axum
- **operator** - Kubernetes operator
- **front** - React/TypeScript frontend (Vite, TailwindCSS, Radix UI)

## Development Commands

### Backend (Rust)

```bash
# Database setup (required first time)
cd core
DATABASE_URL=postgres://ferriskey:ferriskey@localhost:5432/ferriskey sqlx migrate run

# Run API server (from api/)
cd ../api
cargo run

# Run all tests
cargo test

# Run tests for specific package
cargo test -p ferriskey-core
cargo test -p ferriskey-api

# Run one integration suite (requires PostgreSQL; every such test is #[ignore]d)
cargo test -p ferriskey-api --test <feature>_test -- --ignored

# Check code (faster than build)
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

### Frontend (React)

```bash
cd front

# Install dependencies
pnpm install

# Dev server (port 5555)
pnpm run dev

# Build for production
pnpm run build

# Lint
pnpm run lint

# Preview production build
pnpm run preview
```

### Docker

```bash
# Full stack with local build
docker compose --profile local up -d

# Full stack with registry image
docker compose --profile registry up -d
```

## Architecture

### Hexagonal Architecture (Ports & Adapters)

The `core` crate strictly follows hexagonal architecture:

```
core/src/
├── domain/           # Pure business logic (no infrastructure)
│   ├── authentication/
│   ├── user/
│   ├── client/
│   ├── realm/
│   ├── credential/
│   ├── role/
│   ├── trident/      # MFA system
│   ├── seawatch/     # Audit logging
│   ├── webhook/
│   ├── jwt/
│   └── common/       # Shared domain types
├── application/      # Application services (orchestration)
├── infrastructure/   # Concrete implementations
│   ├── repositories/ # Database access via SeaORM
│   └── db/postgres/  # Connection & migrations
└── entity/          # SeaORM auto-generated entities
```

**Key Pattern:**
Each domain module contains:
- `entities.rs` - Domain value objects (immutable)
- `ports.rs` - Trait definitions (repository/service contracts)
- `services.rs` - Business logic implementation
- `value_objects.rs` - DTOs for use cases
- `policies.rs` - Authorization policies (where applicable)

**Dependency Flow:**
- Domain depends on nothing
- Application depends on domain (uses ports)
- Infrastructure implements ports
- `ApplicationService` in `application/mod.rs` composes everything via dependency injection

### API Layer (`libs/ferriskey-api-*`)

Built with Axum. Each feature is its own crate (#1157); `api/` is a thin binary that composes them.

```
api/src/application/http/server/
├── http_server.rs      # merges every feature router
├── app_state.rs        # application state (contains services)
└── openapi.rs          # aggregates every feature's ApiDoc, Swagger/ReDoc/Scalar

libs/
├── ferriskey-api-core/         # shared: AppState, ApiError, auth middleware, extractors
├── ferriskey-api-account/      # self-service account (/users/me)
├── ferriskey-api-authentication/
├── ferriskey-api-user/
├── ferriskey-api-client/
├── ferriskey-api-realm/
├── ferriskey-api-trident/      # MFA
├── ferriskey-api-seawatch/     # security events
└── …                           # one per feature, see the workspace members in Cargo.toml
```

**Crate pattern:** each exposes `<feature>_routes(state: AppState) -> Router<AppState>` and a
`<Feature>ApiDoc` (utoipa `#[derive(OpenApi)]`), laid out as `src/router.rs`, `src/handlers/`,
`src/validators.rs`, `src/errors.rs`.

**Adding a feature crate** takes three registration points: workspace members in the root
`Cargo.toml`, a `.merge(...)` in `http_server.rs`, and a `nest(...)` in `openapi.rs`.

**Handler pattern:**
- Handlers take `State<AppState>` for the services and `Extension<Identity>` for the caller
- Errors convert: `CoreError` → `ApiError` (`libs/ferriskey-api-core/src/error.rs`) — that match is
  exhaustive, so a new `CoreError` variant does not compile until it is mapped
- OpenAPI docs via `utoipa` attributes on handlers; paths are relative to the crate's `nest` prefix

### Frontend (`front/`)

React 19 + TypeScript + Vite stack:

```
src/
├── api/                 # API client layer
│   ├── api.client.ts   # Auto-generated from OpenAPI
│   ├── api.tanstack.ts # React Query wrappers
│   └── *.api.ts        # Feature-specific calls
├── pages/              # Feature pages (match backend modules)
├── components/         # Reusable UI (Radix UI primitives)
├── hooks/              # Custom hooks
├── store/              # Zustand state (auth, user, realm)
├── types/              # TypeScript definitions
└── routes/             # React Router v7
```

**State Management:**
- **Server state:** React Query (TanStack Query)
- **Client state:** Zustand (auth, realm context)
- **Forms:** react-hook-form + Zod validation
- **Frontend API calls:** Prefer the generated `createApiClient` / `window.tanstackApi` client and TanStack Query hooks or mutations over direct `axios` calls. Use `axios` only when a flow cannot be expressed through the generated client.

## Database & Migrations

- **PostgreSQL** via SeaORM (async ORM)
- Migrations in `core/migrations/` (timestamped SQL files)
- Entities auto-generated in `core/src/entity/` (do not edit manually)

**Running migrations:**
```bash
cd core
DATABASE_URL=postgres://ferriskey:ferriskey@localhost:5432/ferriskey sqlx migrate run
```

**Generating entities after migration:**
```bash
# Install sea-orm-cli if not present
cargo install sea-orm-cli

# Generate from database
# --expanded-format is REQUIRED: without it sea-orm-cli emits the compact
# DeriveEntityModel layout, while every entity in this repo is the expanded one.
# Generate into a throwaway database and directory, then copy only the new files:
# regenerating in place rewrites entities that have accumulated hand edits.
sea-orm-cli generate entity \
  --expanded-format \
  --max-connections 1 \
  --database-url postgres://ferriskey:ferriskey@localhost:5432/<throwaway-db> \
  --output-dir /tmp/entitygen
```

`mod.rs` and `prelude.rs` are edited by hand afterwards — the generated ones do not match
what the repo declares.

## Testing Strategy

**Unit Tests:**
- Embedded in source files with `#[test]` or `#[tokio::test]`
- Domain services use `mockall` for repository mocking
- Infrastructure tests validate implementations (e.g., Argon2, recovery codes)

**Integration Tests:**
- One standalone file per feature: `api/tests/<feature>_test.rs`. There is no `api/tests/it/`.
- They need a running PostgreSQL, so every test is `#[ignore]`d and does not block `cargo test`.
- Each suite provisions its own schema and acts through its own dedicated user, so suites stay
  parallel-safe.
- Build the router **once** per suite behind a `OnceLock<SharedContext>`. Building it per test
  panics with `Failed to set global recorder`: the Prometheus recorder is global to the process.
- `axum-test` drives the HTTP endpoints.
- A new suite must be added to `.github/workflows/integration-tests.yaml` — a suite that exists and
  never runs reads as coverage it does not provide.

**Run tests:**
```bash
# unit tests, the whole workspace
cargo test --workspace --lib

# one integration suite (needs PostgreSQL)
cargo test -p ferriskey-api --test <feature>_test -- --ignored

# with output
cargo test -- --nocapture
```

**Before pushing**, run what CI runs — `pre-commit` fails on formatting before it ever lints:
```bash
cargo fmt --all -- --check
cargo clippy --all --all-targets --exclude ferriskey-client -- -D warnings
cargo check --all-targets
```

## Key Domain Concepts

**Multi-Tenancy (Realms):**
- Every user, client, role, credential belongs to a realm
- Complete isolation between realms
- Realm-specific configuration and settings

**Bitwise Role System:**
- Roles are bitmasks for efficient permission checks
- User/client roles stored as integers
- Quick AND operations for authorization

**Required Actions:**
- Users can have required actions (e.g., MFA setup, password reset)
- Enforced before full authentication completes
- Handled in `domain/user/required_actions.rs`

**Trident Module (MFA):**
- TOTP (Time-based One-Time Password)
- WebAuthn (passwordless with security keys)
- Magic Links (email-based)
- Recovery Codes (backup codes)

**SeaWatch Module (Audit):**
- Security event logging for all critical actions
- Queryable event trail
- Exportable audit logs

**Webhooks:**
- Event-driven extensibility
- Subscribe to lifecycle events (user created, client updated, etc.)
- Async delivery to configured endpoints

## Error Handling

Layered error types:
1. `CoreError` (domain) - Business logic errors in `core/src/domain/common/errors.rs`
2. `ApiError` (HTTP) - HTTP response format in `api/src/application/http/errors/error.rs`
3. Automatic conversion via `From<CoreError> for ApiError`

Errors are traced through the system with context.

## Configuration

- **Clap** for CLI args and environment variables
- Main config in `api/src/Args` struct
- Environment variables override defaults
- Copy `.env.example` to `.env` in `api/` directory

**Key variables:**
```bash
DATABASE_URL=postgres://ferriskey:ferriskey@localhost:5432/ferriskey
PORT=3333
ADMIN_USERNAME=admin
ADMIN_PASSWORD=admin
ADMIN_EMAIL=admin@ferriskey.rs
ALLOWED_ORIGINS=http://localhost:5555
LOG_LEVEL=info
```

## Observability

- **Metrics:** Prometheus endpoint at `/metrics` (via `axum-prometheus`)
- **Tracing:** Structured logging with `tracing` crate
- **OpenTelemetry:** Optional OTLP integration for distributed tracing
- **OpenAPI Docs:** Available at `/swagger-ui`, `/redoc`, `/rapidoc`, `/scalar`

## Common Patterns

**Adding a new domain entity:**
1. Create migration in `core/migrations/`
2. Run migration and regenerate entities
3. Create domain module in `core/src/domain/your_feature/`
4. Define entities, ports, services, value_objects
5. Implement repository in `core/src/infrastructure/repositories/`
6. Wire up in `ApplicationService` (`core/src/application/mod.rs`)
7. Add HTTP handlers in `api/src/application/http/your_feature/`
8. Add routes to main router in `api/src/application/http/server/http_server.rs`

**Repository pattern:**
All repositories are traits defined in `domain/*/ports.rs`, implemented in `infrastructure/repositories/*_repository.rs`. Services depend on traits, not concrete implementations (enables mocking).

**Generic services:**
Domain services use generic type parameters for repositories (e.g., `UserService<R: UserRepository>`). Type aliases in `application/services.rs` provide concrete types.

**Error propagation:**
Use `?` operator liberally. Errors convert automatically via `From` implementations. Add context with `.map_err()` where helpful.

## CI/CD

GitHub Actions workflows in `.github/workflows/`:
- `docker.yaml` - Builds and pushes Docker images on main/tags
- `helm.yaml` - Packages and publishes Helm charts
- `pre-commit.yaml` - Runs pre-commit hooks
- `release.yaml` - Creates GitHub releases

## Important Notes

- **Never edit** `core/src/entity/` files manually - they're generated from migrations
- **Async everywhere** - Use `tokio::test` for async tests, `.await` on all DB/HTTP calls
- **Type safety** - Leverage Rust's type system; prefer compile-time errors over runtime
- **No unwrap()** - `.unwrap()`/`.expect()` belong in tests and provably-infallible cases only
- **Cookie handling** - Uses `axum-extra` for cookie management
- **Default credentials** - admin/admin for local development (change in production)

## Useful Links

- Documentation: https://docs.ferriskey.rs/getting-started/introduction
- Helm Chart: `oci://ghcr.io/ferriskey/charts/ferriskey`
- Discussions: https://github.com/ferriskey/ferriskey/discussions
