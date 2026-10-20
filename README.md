<p align="center">
  <img src="./front/public/logo_ferriskey.png" alt="FerrisKey logo" width="100" />
</p>

<p align="center">
  <strong>FerrisKey</strong><br/>
  Open-source identity and access management, written in Rust
</p>

<p align="center">
  <a href="https://github.com/ferriskey/ferriskey/actions">
    <img alt="CI" src="https://img.shields.io/github/actions/workflow/status/ferriskey/ferriskey/docker.yaml?label=CI&logo=github" />
  </a>
  <a href="https://github.com/ferriskey/ferriskey/releases">
    <img alt="Release" src="https://img.shields.io/github/v/release/ferriskey/ferriskey?display_name=tag&logo=semantic-release" />
  </a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/ferriskey/ferriskey">
    <img alt="OpenSSF Scorecard" src="https://api.scorecard.dev/projects/github.com/ferriskey/ferriskey/badge" />
  </a>
  <a href="https://www.bestpractices.dev/projects/11019">
    <img alt="OpenSSF Best Practices" src="https://www.bestpractices.dev/projects/11019/badge" />
  </a>
  <a href="https://landscape.cncf.io/?item=provisioning--security-compliance--ferriskey">
    <img alt="CNCF Landscape" src="https://img.shields.io/badge/CNCF%20Landscape-5699C6" />
  </a>
  <a href="https://opensource.org/licenses/Apache-2.0">
    <img alt="License" src="https://img.shields.io/badge/License-Apache_2.0-blue.svg" />
  </a>
  <a href="https://github.com/ferriskey/ferriskey/stargazers">
    <img alt="Stars" src="https://img.shields.io/github/stars/ferriskey/ferriskey?logo=github" />
  </a>
  <a href="https://github.com/sponsors/ferriskey">
    <img alt="Sponsor" src="https://img.shields.io/badge/Sponsor-❤-ff69b4?logo=github-sponsors" />
  </a>
</p>

<a id="sponsors"></a>
<p align="center"><sub>Sponsors</sub></p>
<p align="center">
  <a href="https://cloud-iam.com">
    <img src="./docs/logos/cloud-iam.svg" alt="Cloud-IAM" height="40">
  </a>
  &nbsp;&nbsp;&nbsp;
  <a href="https://france-nuage.fr">
    <img src="./docs/logos/france-nuage.png" alt="France Nuage" height="40">
  </a>
  &nbsp;&nbsp;&nbsp;
  <a href="https://www.gilded.ch/">
    <img src="./docs/logos/gilded_health.svg" alt="Gilded Health" height="40">
  </a>
  &nbsp;&nbsp;&nbsp;
  <a href="https://www.nudibranches.tech/">
    <img src="./docs/logos/nudibranches.png" alt="Nudibranches" height="40">
  </a>
</p>

---

## Why FerrisKey?

FerrisKey is an identity and access management (IAM) server written in Rust. It implements OpenID Connect and OAuth 2.0, keeps the users, roles and clients of each tenant in a separate realm, and ships with an admin console and a Helm chart. It is meant as an open-source alternative to the heavier IAM servers.

## Table of contents

- [Features](#features)
- [Quick start](#quick-start)
- [Configuration](#configuration)
- [Database migrations](#database-migrations)
- [Modules](#modules)
- [Architecture](#architecture)
- [Observability](#observability)
- [Roadmap](#roadmap)
- [Adopters](#adopters)
- [Contributing](#contributing)
- [Code of conduct](#code-of-conduct)
- [Security](#security)
- [Sponsors and governance](#sponsors-and-governance)
- [License](#license)
- [Links](#links)

<a id="features"></a>

## Features

| Capability                   | Details |
|------------------------------|---|
| OpenID Connect / OAuth 2.0   | Standard flows for applications and services. |
| Multi-tenant realms          | Users, roles, clients and secrets are isolated per realm. |
| Clients and service accounts | Fine-grained role mapping on a bitwise role system. |
| MFA                          | TOTP, WebAuthn, magic links and recovery codes, enforced through required actions. |
| Observability                | Prometheus metrics and a security event log. |
| Kubernetes                   | Helm chart published as an OCI artifact. |

<a id="quick-start"></a>

## Quick start

For local development with the repository `justfile`, see [DEVELOPMENT.md](./DEVELOPMENT.md).

### Option A: published Docker images

```bash
docker compose --profile registry up -d
```

### Option B: build the Docker images

```bash
docker compose --profile build up -d
```

With either option, the console is at <http://localhost:5555>. The default credentials are `admin` / `admin`.

> The compose file mounts `pgdata` at `/var/lib/postgresql`, the mount point recommended from Postgres 18. If an older local setup used a different path, remove or migrate the old volume first. `docker compose down -v` resets it and deletes its data.

### Option C: Helm (Kubernetes)

The chart needs a reachable Postgres, external or deployed by your platform's operator. See the [chart documentation](charts/ferriskey/README.md).

### Option D: Cargo

1. Clone the repository.

   ```bash
   git clone https://github.com/ferriskey/ferriskey
   cd ferriskey
   ```

2. Start the database and run the migrations.

   ```bash
   cd api
   cp env.example .env
   # edit .env if the defaults do not suit you
   docker compose up -d
   cd ../core
   # sqlx-cli: cargo install sqlx-cli --no-default-features --features postgres
   DATABASE_URL=postgres://ferriskey:ferriskey@localhost:5432/ferriskey sqlx migrate run
   ```

3. Start the API. It needs a recent stable Rust toolchain (`rustup update`).

   ```bash
   cd ../api
   cargo run
   ```

4. Start the frontend (optional).

   ```bash
   cd ../front
   cp .env.example .env
   pnpm install
   pnpm run dev
   ```

The console is at <http://localhost:5555>. The default credentials are `admin` / `admin`.

<a id="configuration"></a>

## Configuration

Common environment variables:

```
PORT=3333
ENV=development
LOG_LEVEL=info
DATABASE_URL=postgres://ferriskey:ferriskey@localhost:5432/ferriskey

ADMIN_PASSWORD=admin
ADMIN_USERNAME=admin
ADMIN_EMAIL=admin@ferriskey.rs

ALLOWED_ORIGINS=http://localhost:5555
```

`ALLOWED_ORIGINS` lists the origins accepted on every route. Each client can also declare its own web origins at runtime, enforced per realm (see the `web-origins` endpoints of a client). Those per-client origins never replace `ALLOWED_ORIGINS`: `/config`, the health probes and the API docs carry no realm, so an admin console served from another origin than the API must keep its origin in `ALLOWED_ORIGINS`.

By default, the API listens on port 3333 and the frontend on port 5555.

<a id="database-migrations"></a>

## Database migrations

FerrisKey uses SQLx for database migrations. To apply pending migrations:

```bash
cd core
DATABASE_URL=postgres://ferriskey:ferriskey@localhost:5432/ferriskey sqlx migrate run
```

To check migration status:

```bash
DATABASE_URL=postgres://ferriskey:ferriskey@localhost:5432/ferriskey sqlx migrate info
```

If `DATABASE_URL` is set in your environment or in `.env`, you can drop it from the command.

<a id="modules"></a>

## Modules

- Trident: multi-factor authentication (TOTP, WebAuthn, magic links, recovery codes) and the required actions that enforce it.
- SeaWatch: security event log, queryable and exportable from the console.
- Webhooks: notifications on user, client and realm lifecycle events, so integrations do not need a fork.

<a id="architecture"></a>

## Architecture

FerrisKey follows a hexagonal (ports and adapters) architecture. The domain logic does not depend on the database or the HTTP layer: those implement the ports the domain defines, and can be replaced without touching it.

<a id="observability"></a>

## Observability

The API serves Prometheus metrics on `/metrics`, under the configured root path.

<a id="roadmap"></a>

## Roadmap

The public roadmap is at <https://ferriskey.rs/roadmap>. [ROADMAP.md](./ROADMAP.md) lists the epics in progress and how the roadmap is maintained.

<a id="adopters"></a>

## Adopters

Organizations using FerrisKey are listed in [ADOPTERS.md](./ADOPTERS.md). If you run FerrisKey, add yourself with a pull request.

<a id="contributing"></a>

## Contributing

Bug fixes, features, documentation and tests are all welcome.

1. Read [CONTRIBUTING.md](./CONTRIBUTING.md).
2. Pick an issue. Those labelled `good first issue` are a good place to start.
3. Open a pull request with tests and a short description.

Questions about modules, APIs or the console go to [Discussions](https://github.com/ferriskey/ferriskey/discussions).

<a id="code-of-conduct"></a>

## Code of conduct

FerrisKey follows the [CNCF Code of Conduct](https://github.com/cncf/foundation/blob/main/code-of-conduct.md). See [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md) for scope and how to report an incident.

<a id="security"></a>

## Security

Report vulnerabilities privately through the [Security tab](https://github.com/ferriskey/ferriskey/security/advisories/new), never in a public issue. Supported versions, response targets and the disclosure process are in [SECURITY.md](./SECURITY.md).

<a id="sponsors-and-governance"></a>

## Sponsors and governance

FerrisKey is Apache-2.0 only, with no CLA, no open-core model and no enterprise edition. Sponsors fund the project but do not control it. See [Sponsors and open source commitments](./GOVERNANCE.md#sponsors-and-open-source-commitments) in GOVERNANCE.md.

The project is preparing a CNCF Sandbox application, tracked in [#1623](https://github.com/ferriskey/ferriskey/issues/1623). The answers to the Day 0 General Technical Review live in [docs/cncf/general-technical-review.md](./docs/cncf/general-technical-review.md).

<a id="license"></a>

## License

FerrisKey is released under the [Apache-2.0 license](./LICENSE).

<a id="links"></a>

## Links

- Source: <https://github.com/ferriskey/ferriskey>
- Helm chart (OCI): `oci://ghcr.io/ferriskey/charts/ferriskey`
- Documentation: <https://docs.ferriskey.rs/en/discover/what-is-ferriskey>
- Discussions: <https://github.com/ferriskey/ferriskey/discussions>
- Sponsor: <https://github.com/sponsors/ferriskey>
