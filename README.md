<!-- PROJECT BANNER -->
<p align="center">
  <img src="./front/public/logo_ferriskey.png" alt="FerrisKey — Modern Open‑Source IAM in Rust" width="100" />
</p>

<p align="center">
  <strong>FerrisKey</strong> — Open‑Source, High‑Performance Identity & Access Management<br/>
  <em>Cloud‑native • Extensible • Built in Rust</em>
</p>

<p align="center">
  <!-- Badges (tweak org/repo names as needed) -->
  <a href="https://github.com/ferriskey/ferriskey/actions">
    <img alt="CI" src="https://img.shields.io/github/actions/workflow/status/ferriskey/ferriskey/ci.yml?label=CI&logo=github" />
  </a>
  <a href="https://github.com/ferriskey/ferriskey/releases">
    <img alt="Release" src="https://img.shields.io/github/v/release/ferriskey/ferriskey?display_name=tag&logo=semantic-release" />
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

---

## ✨ Why FerrisKey?

FerrisKey is a modern **Identity & Access Management (IAM)** platform built with **Rust** and a **hexagonal architecture**.
It aims to be a serious open‑source alternative to heavyweight IAMs fast, modular, and cloud‑native by design.

- 🦀 **Performance-first** — Rust, async I/O, low latency.
- 🧱 **Hexagonal architecture** — clean domain, clear ports/adapters.
- 🏢 **Multi‑tenant realms** — strong isolation of users/roles/clients.
- 🔐 **Modern auth** — OIDC/OAuth2, MFA (TOTP).
- 🧩 **Extensibility** — native modules for MFA, auditability, and webhooks.
- ☁️ **Cloud‑native** — official Helm chart; ready for Kubernetes.


## 🧭 Table of Contents

- [Features](#-features)
- [Quick Start](#-quick-start)
- [Configuration](#-configuration)
- [Modules](#-modules)
- [Architecture](#-architecture)
- [Observability](#-observability)
- [Roadmap](#-roadmap)
- [Contributing](#-contributing)
- [Security](#-security)
- [License](#-license)
- [Links](#-links)


## 🌟 Features

| Capability                      | Details |
|---------------------------------|---|
| **OIDC / OAuth2**               | Standards‑compliant flows for modern apps & services. |
| **Multi‑Tenant Realms**         | Logical isolation of users, roles, clients, secrets. |
| **Clients & Service Accounts**  | Fine‑grained role mapping; bitwise role system. |
| **MFA (TOTP)**                  | Pluggable strategies with required actions. |
| **Observability**               | Prometheus metrics, Grafana dashboards. |
| **Kubernetes‑ready**            | Helm chart with sane defaults; OCI distribution. |

> **License:** Apache‑2.0. No paywalls. Community‑first.

## 🚀 Quick Start

### Option A — Docker (Docker compose)

```yaml
services:
  postgres:
    image: docker.io/postgres:17
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=postgres
      - POSTGRES_DB=ferriskey
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped
  api-migration:
    image: ghcr.io/ferriskey/ferriskey-api:latest
    environment:
      - DATABASE_URL=postgres://postgres:postgres@postgres:5432/ferriskey
    depends_on:
      - postgres
    command: >
      bash -c "
        sqlx migrate run &&
        echo 'Database migrations completed!'
      "
    restart: "no"
  api:
    image: ghcr.io/ferriskey/ferriskey-api:latest
    environment:
      - PORT=3333
      - DATABASE_URL=postgres://postgres:postgres@postgres:5432/ferriskey
      - ADMIN_EMAIL=admin@example.com
      - ADMIN_PASSWORD=admin
      - ADMIN_USERNAME=admin
      - ALLOWED_ORIGINS=http://localhost:5555
    depends_on:
      api-migration:
        condition: service_completed_successfully
    ports:
      - "3333:3333"
    restart: unless-stopped
  frontend:
    image: ghcr.io/ferriskey/ferriskey-front:latest
    ports:
      - "5555:80"
    environment:
      - APP_API_URL=http://localhost:3333
    depends_on:
      - api
volumes:
  postgres_data:
```

Then visit [http://localhost:5555](http://localhost:5555) to access the console. The default credentials are `admin` and `admin`.

### Option B — Helm (Kubernetes)
> Requires a reachable Postgres (or include it via your platform’s recommended operator).

```bash

helm upgrade --install ferriskey oci://ghcr.io/ferriskey/charts/ferriskey \
  --namespace ferriskey --create-namespace \
  --set api.monitoring.serviceMonitor.enabled=false
```

### Option C - Cargo

1. Clone the repo
```bash
git clone https://github.com/ferriskey/ferriskey
```

2. Launch the database and execute migrations with sourced env variables

```bash
cd api
cp env.example .env
# feel free to change the env variables in .env to your liking.
docker compose up -d
cd ../core
# to install sqlx you might need to run `cargo install sqlx-cli`
DATABASE_URL=postgres://postgres:postgres@localhost:5432/ferriskey sqlx migrate run
```
3. Launch the API

```bash
cd ../api
cargo run
```

4. Launch the frontend (optional)

```bash
cd ../front
source env.sh
pnpm install
pnpm run dev
```

Then visit [http://localhost:5555](http://localhost:5555) to access the console. The default credentials are `admin` and `admin`.

## ⚙️ Configuration
Common environment variables (example):

```
PORT=3333
ENV=development
LOG_LEVEL=info
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/ferriskey

ADMIN_PASSWORD=admin
ADMIN_USERNAME=admin
ADMIN_EMAIL=admin@ferriskey.rs

ALLOWED_ORIGINS=http://localhost:5555
```

By default, the API will listen on port 3333 and the frontend on port 5555.

## 🧩 Modules
- Trident — MFA & security scopes
TOTP, WebAuthn, Magic Link; flexible required actions.

- SeaWatch — Observability & audit logs
Security event trails; queryable from the console; exportable.

- Webhooks — Event‑driven extensibility
Subscribe to user/client/realm lifecycle events without forking core.



## 🏗️ Architecture
FerrisKey follows a Hexagonal Architecture (Ports & Adapters) to keep business logic pure and infrastructure replaceable.



## 📈 Observability
- Metrics: /metrics (Prometheus format)
- Dashboards: Starter Grafana dashboards included in Helm values (optional)

## 🤝 Contributing
We welcome contributions of all kinds bugfixes, features, docs, testing.
1. Read [CONTRIBUTING.md](./CONTRIBUTING.md)
2. Pick an issue (good first issues labelled)
3. Open a PR with tests and a concise description
> Join discussions to help shape modules, APIs, and UX.

## 🔐 Security
Please report vulnerabilities responsibly via Security Advisories.
Avoid filing publicly until coordinated disclosure is agreed.



## 📜 License
Apache‑2.0 — free to use, modify, and distribute.

## 🔗 Links
- 📂 Source: https://github.com/ferriskey/ferriskey
- 📦 Helm Chart (OCI): `oci://ghcr.io/ferriskey/charts/ferriskey`
- 📖 Documentation: https://ferriskey.rs/docs/welcome/introduction
- 💬 Discussions: https://github.com/ferriskey/ferriskey/discussions
- 🏆 Sponsor: https://github.com/sponsors/ferriskey
