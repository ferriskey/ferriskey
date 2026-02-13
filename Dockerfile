ARG RUST_VERSION=1.91.1
ARG NODE_VERSION=20.14.0
ARG ALPINE_VERSION=3.21
ARG RUNTIME_BASE=cgr.dev/chainguard/wolfi-base:latest

FROM rust:${RUST_VERSION}-bookworm AS rust-build

WORKDIR /usr/local/src/ferriskey

RUN \
  apt-get update && \
  apt-get install -y --no-install-recommends ca-certificates && \
  rm -rf /var/lib/apt/lists/*

RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  cargo install \
  --root /usr/local/sqlx \
  sqlx-cli \
  --no-default-features \
  --features postgres,rustls

COPY Cargo.toml Cargo.lock ./
COPY libs/maskass/Cargo.toml ./libs/maskass/
COPY libs/ferriskey-domain/Cargo.toml ./libs/ferriskey-domain/
COPY libs/ferriskey-security/Cargo.toml ./libs/ferriskey-security/
COPY libs/ferriskey-trident/Cargo.toml ./libs/ferriskey-trident/
COPY libs/ferriskey-abyss/Cargo.toml ./libs/ferriskey-abyss/
COPY core/Cargo.toml ./core/
COPY api/Cargo.toml ./api/
COPY operator/Cargo.toml ./operator/

RUN \
  mkdir -p \
  api/src \
  core/src \
  operator/src \
  libs/maskass/src \
  libs/ferriskey-domain/src \
  libs/ferriskey-security/src \
  libs/ferriskey-trident/src \
  libs/ferriskey-abyss/src && \
  touch libs/maskass/src/lib.rs && \
  touch libs/ferriskey-domain/src/lib.rs && \
  touch libs/ferriskey-security/src/lib.rs && \
  touch libs/ferriskey-trident/src/lib.rs && \
  touch libs/ferriskey-abyss/src/lib.rs && \
  touch core/src/lib.rs && \
  printf "fn main() {}\n" > api/src/main.rs && \
  printf "fn main() {}\n" > operator/src/main.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/local/src/ferriskey/target \
  cargo build --release

COPY libs/maskass libs/maskass
COPY libs/ferriskey-domain libs/ferriskey-domain
COPY libs/ferriskey-security libs/ferriskey-security
COPY libs/ferriskey-trident libs/ferriskey-trident
COPY libs/ferriskey-abyss libs/ferriskey-abyss
COPY core core
COPY api api
COPY operator operator

RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  cargo build --release

FROM ${RUNTIME_BASE} AS runtime

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

USER nonroot

FROM runtime AS api

COPY --from=rust-build /usr/local/src/ferriskey/target/release/ferriskey-api /usr/local/bin/ferriskey-api
COPY --from=rust-build /usr/local/src/ferriskey/core/migrations /usr/local/src/ferriskey/migrations
COPY --from=rust-build /usr/local/sqlx/bin/sqlx /usr/local/bin/sqlx

EXPOSE 80

ENTRYPOINT ["/usr/local/bin/ferriskey-api"]

FROM runtime AS operator

COPY --from=rust-build /usr/local/src/ferriskey/target/release/ferriskey-operator /usr/local/bin/ferriskey-operator

EXPOSE 80

ENTRYPOINT ["/usr/local/bin/ferriskey-operator"]

FROM node:${NODE_VERSION}-alpine AS webapp-build

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

RUN pnpm run build

FROM nginx:1.28.0-alpine${ALPINE_VERSION}-slim AS webapp

COPY --from=webapp-build /usr/local/src/ferriskey/dist /usr/local/src/ferriskey
COPY front/nginx.conf /etc/nginx/conf.d/default.conf
COPY front/docker-entrypoint.sh /docker-entrypoint.d/docker-entrypoint.sh

RUN chmod +x /docker-entrypoint.d/docker-entrypoint.sh
