FROM rust:1.99.0-trixie@sha256:81015672d6939b0265f09adf96d5de125482bb1330106076956b0518e74edc90 AS chef

WORKDIR /usr/local/src/ferriskey

RUN cargo install cargo-chef --version 0.1.77 --locked && \
    cargo install sqlx-cli --version 0.8.6 --no-default-features --features postgres --locked

# ── Planner: analyse the workspace and produce recipe.json ────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Builder: cook deps first (cached), then build real source ─────────────────
FROM chef AS builder

COPY --from=planner /usr/local/src/ferriskey/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
# Only these two binaries are shipped; skip the rest of the workspace.
RUN cargo build --release --bin ferriskey-api --bin ferriskey-operator

# ── Shared runtime base ───────────────────────────────────────────────────────
# distroless/cc ships glibc, libgcc_s, libssl3/libcrypto3 and ca-certificates —
# exactly the dynamic dependencies of our binaries — in ~34 MB, where
# debian:bookworm-slim plus the equivalent apt packages costs ~106 MB.
# It has no shell and no package manager: every entrypoint used against these
# images (`ferriskey-api`, `ferriskey-operator`, `sqlx migrate run`) is an exec
# of a binary on PATH, so nothing depends on /bin/sh.
FROM gcr.io/distroless/cc-debian13:nonroot@sha256:b7e81cb1af38d9345680e68bcf564d388f96304e25222030fc8c051faf8565b3 AS runtime

# distroless defaults WORKDIR to /home/nonroot; keep / as before.
WORKDIR /

# ── API image ─────────────────────────────────────────────────────────────────
FROM runtime AS api

COPY --from=builder /usr/local/src/ferriskey/target/release/ferriskey-api /usr/local/bin/
COPY --from=builder /usr/local/src/ferriskey/core/migrations /usr/local/src/ferriskey/migrations
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/

EXPOSE 80

ENTRYPOINT ["ferriskey-api"]

# ── Operator image ────────────────────────────────────────────────────────────
FROM runtime AS operator

COPY --from=builder /usr/local/src/ferriskey/target/release/ferriskey-operator /usr/local/bin/

EXPOSE 80

ENTRYPOINT ["ferriskey-operator"]

# ── Frontend build ────────────────────────────────────────────────────────────
FROM node:26.12.0-alpine3.24@sha256:6f08a523443ddedf52941874c203475b0ad503e859ad70cb26ab8be9a517dbd5 AS webapp-build

WORKDIR /usr/local/src/ferriskey

ENV PNPM_HOME="/pnpm"
ENV PATH="/opt/pnpm/node_modules/.bin:$PNPM_HOME:$PATH"

COPY tools/pnpm/package.json tools/pnpm/package-lock.json /opt/pnpm/

RUN npm ci --prefix /opt/pnpm --ignore-scripts

COPY front/package.json front/pnpm-lock.yaml front/pnpm-workspace.yaml ./

RUN pnpm install --frozen-lockfile

COPY front/ .

RUN VITE_API_URL="" pnpm run build

# ── Frontend runtime ──────────────────────────────────────────────────────────
FROM nginxinc/nginx-unprivileged:1.31.2-alpine3.23-slim@sha256:eed3e4c2f3f9285e80610238e0b18fe1bf311c50559434db1969b1cfabd5518b AS webapp

COPY --chown=101:101 front/nginx.conf /etc/nginx/conf.d/default.conf
# Owned by the nginx user (101) so the entrypoint can write API_URL into config.json.
COPY --chown=101:101 --from=webapp-build /usr/local/src/ferriskey/dist /usr/share/nginx/html
COPY --chmod=0755 front/docker-entrypoint.sh /docker-entrypoint.d/docker-entrypoint.sh

# ── Standalone image (API + Frontend, single container) ───────────────────────
FROM debian:trixie-slim@sha256:62bd423aeee912639ca8ee681c2b762cb3e25137e0a92b20be455898a9f034ad AS standalone

# hadolint ignore=DL3008
RUN \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates=20250419 \
    libssl3t64=3.5.7-1~deb13u3 \
    nginx \
    supervisor && \
    rm -rf /var/lib/apt/lists/* && \
    rm -f /etc/nginx/sites-enabled/default

COPY --from=builder /usr/local/src/ferriskey/target/release/ferriskey-api /usr/local/bin/
COPY --from=builder /usr/local/src/ferriskey/core/migrations /usr/local/src/ferriskey/migrations
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/
# Owned by the nginx user (101) so the entrypoint can write API_URL into config.json.
COPY --chown=101:101 --from=webapp-build /usr/local/src/ferriskey/dist /usr/share/nginx/html
COPY --chmod=0755 front/docker-entrypoint.sh /docker-entrypoint.d/docker-entrypoint.sh

COPY front/nginx-standalone.conf /etc/nginx/conf.d/default.conf
COPY docker/supervisord.conf /etc/supervisor/conf.d/ferriskey.conf
COPY --chmod=0755 docker/standalone-entrypoint.sh /standalone-entrypoint.sh

EXPOSE 80

ENTRYPOINT ["/standalone-entrypoint.sh"]
