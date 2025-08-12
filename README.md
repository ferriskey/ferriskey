<!-- PROJECT BANNER -->
<p align="center">
  
</p>

<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="400" viewBox="0 0 1600 400">
  <defs>
    <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#0ea5e9"/>
      <stop offset="50%" stop-color="#6366f1"/>
      <stop offset="100%" stop-color="#22c55e"/>
    </linearGradient>
    <linearGradient id="glow" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0%" stop-color="rgba(255,255,255,0.0)"/>
      <stop offset="50%" stop-color="rgba(255,255,255,0.35)"/>
      <stop offset="100%" stop-color="rgba(255,255,255,0.0)"/>
    </linearGradient>
    <filter id="soft">
      <feGaussianBlur stdDeviation="18" />
    </filter>
  </defs>

  <rect width="1600" height="400" fill="url(#g)"/>
  <g opacity="0.25">
    <circle cx="200" cy="100" r="180" fill="white" filter="url(#soft)"/>
    <circle cx="1200" cy="320" r="220" fill="white" filter="url(#soft)"/>
  </g>

  <g>
    <text x="100" y="220" font-family="ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial" font-size="80" font-weight="800" fill="#ffffff">
      🦀 FerrisKey
    </text>
    <text x="100" y="270" font-family="ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial" font-size="26" fill="#e6f0ff">
      Open‑Source, High‑Performance IAM • Rust • Cloud‑native • Extensible
    </text>
  </g>

  <rect x="100" y="310" width="1400" height="4" fill="url(#glow)" opacity="0.75"/>
</svg>

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
- 🔐 **Modern auth** — OIDC/OAuth2, MFA (TOTP, WebAuthn, Magic Link).
- 🧩 **Extensibility** — native modules for MFA, auditability, and webhooks.
- ☁️ **Cloud‑native** — official Helm chart; ready for Kubernetes.

---

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

---


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
    image: postgres:17
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
      - PORTAL_URL=http://localhost:5555
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

### Option B — Helm (Kubernetes)
> Requires a reachable Postgres (or include it via your platform’s recommended operator).

```bash

helm upgrade --install ferriskey oci://ghcr.io/ferriskey/charts/ferriskey \
  --namespace ferriskey --create-namespace \
  --set api.monitoring.serviceMonitor.enabled=false
```

## ⚙️ Configuration
Common environment variables (example):

```
PORT=3333
ENV=development
LOG_LEVEL=info
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/ferriskey
PORTAL_URL=http://localhost:5555

ADMIN_PASSWORD=admin
ADMIN_USERNAME=admin
ADMIN_EMAIL=admin@ferriskey.rs

ALLOWED_ORIGINS=http://localhost:5555
```


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
1. Read CONTRIBUTING.md
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
- 📦 Helm Chart (OCI): oci://ghcr.io/ferriskey/charts/ferriskey
- 💬 Discussions: https://github.com/ferriskey/ferriskey/discussions
- 🏆 Sponsor: https://github.com/sponsors/ferriskey


