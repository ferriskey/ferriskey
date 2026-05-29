FROM rust:1.95.0-trixie AS chef

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
RUN cargo build --release

# ── API image ─────────────────────────────────────────────────────────────────
FROM gcr.io/distroless/base-debian13:latest AS api

COPY --from=builder /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/x86_64-linux-gnu/libgcc_s.so.1
COPY --from=builder /usr/local/src/ferriskey/target/release/ferriskey-api /usr/local/bin/
COPY --from=builder /usr/local/src/ferriskey/core/migrations /usr/local/src/ferriskey/migrations
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/

EXPOSE 80

ENTRYPOINT ["ferriskey-api"]

# ── Operator image ────────────────────────────────────────────────────────────
FROM gcr.io/distroless/base-debian13:latest AS operator

COPY --from=builder /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/x86_64-linux-gnu/libgcc_s.so.1
COPY --from=builder /usr/local/src/ferriskey/target/release/ferriskey-operator /usr/local/bin/

EXPOSE 80

ENTRYPOINT ["ferriskey-operator"]

# ── Frontend build ────────────────────────────────────────────────────────────
FROM node:20.14.0-alpine AS webapp-build

WORKDIR /usr/local/src/ferriskey

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

RUN \
    corepack enable && \
    corepack prepare pnpm@9.15.0 --activate && \
    apk --no-cache add dumb-init=1.2.5-r3

COPY front/package.json front/pnpm-lock.yaml ./

RUN pnpm install --frozen-lockfile

COPY front/ .

RUN VITE_API_URL="" pnpm run build

# ── Frontend runtime ──────────────────────────────────────────────────────────
FROM nginxinc/nginx-unprivileged:1.31.1-alpine3.23-slim AS webapp

# The unprivileged image ships root-owned index.html/50x.html in the web root and
# runs as the non-root "nginx" user. The runtime entrypoint wipes and repopulates
# that directory, so it must be owned by the runtime user — otherwise the rm fails
# with "Permission denied" and (under `set -e`) nginx never starts.
USER root
RUN rm -rf /usr/share/nginx/html/* && chown -R nginx:nginx /usr/share/nginx/html
USER nginx

COPY --from=webapp-build /usr/local/src/ferriskey/dist /usr/local/src/ferriskey
COPY front/nginx.conf /etc/nginx/conf.d/default.conf
COPY --chmod=0755 front/docker-entrypoint.sh /docker-entrypoint.d/docker-entrypoint.sh

# ── Standalone image (API + Frontend, single container) ───────────────────────
FROM debian:trixie-slim AS standalone

# hadolint ignore=DL3008
RUN \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates=20250419 \
    libssl3=3.5.6-1~deb13u1 \
    nginx \
    supervisor && \
    rm -rf /var/lib/apt/lists/* && \
    rm -f /etc/nginx/sites-enabled/default

COPY --from=builder /usr/local/src/ferriskey/target/release/ferriskey-api /usr/local/bin/
COPY --from=builder /usr/local/src/ferriskey/core/migrations /usr/local/src/ferriskey/migrations
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/
COPY --from=webapp-build /usr/local/src/ferriskey/dist /usr/share/nginx/html

COPY front/nginx-standalone.conf /etc/nginx/conf.d/default.conf
COPY docker/supervisord.conf /etc/supervisor/conf.d/ferriskey.conf
COPY --chmod=0755 docker/standalone-entrypoint.sh /standalone-entrypoint.sh

EXPOSE 80

ENTRYPOINT ["/standalone-entrypoint.sh"]
