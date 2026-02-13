FROM rust:1.91.1-bookworm AS rust-build

WORKDIR /usr/local/src/ferriskey

RUN cargo install sqlx-cli --no-default-features --features postgres

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
  mkdir -p api/src core/src entity/src operator/src libs/maskass/src libs/ferriskey-domain/src libs/ferriskey-security/src libs/ferriskey-trident/src libs/ferriskey-abyss/src && \
  touch libs/maskass/src/lib.rs && \
  touch libs/ferriskey-domain/src/lib.rs && \
  touch libs/ferriskey-security/src/lib.rs && \
  touch libs/ferriskey-trident/src/lib.rs && \
  touch libs/ferriskey-abyss/src/lib.rs && \
  touch core/src/lib.rs && \
  echo "fn main() {}" > operator/src/main.rs && \
  echo "fn main() {}" > api/src/main.rs && \
  cargo build --release

COPY libs/maskass libs/maskass
COPY libs/ferriskey-domain libs/ferriskey-domain
COPY libs/ferriskey-security libs/ferriskey-security
COPY libs/ferriskey-trident libs/ferriskey-trident
COPY libs/ferriskey-abyss libs/ferriskey-abyss

COPY core core
COPY api api
COPY operator operator

RUN \
  touch libs/maskass/src/lib.rs && \
  touch libs/ferriskey-domain/src/lib.rs && \
  touch libs/ferriskey-security/src/lib.rs && \
  touch libs/ferriskey-trident/src/lib.rs && \
  touch libs/ferriskey-abyss/src/lib.rs && \
  touch core/src/lib.rs && \
  touch operator/src/main.rs && \
  cargo build --release

FROM cgr.dev/chainguard/wolfi-base:latest AS runtime

RUN \
  addgroup -S -g 1000 ferriskey && \
  adduser -S -D -H -u 1000 -G ferriskey ferriskey

USER ferriskey

FROM runtime AS api

COPY --from=rust-build /usr/local/src/ferriskey/target/release/ferriskey-api /usr/local/bin/
COPY --from=rust-build /usr/local/src/ferriskey/core/migrations /usr/local/src/ferriskey/migrations
COPY --from=rust-build /usr/local/cargo/bin/sqlx /usr/local/bin/

EXPOSE 80

ENTRYPOINT ["ferriskey-api"]

FROM runtime AS operator

COPY --from=rust-build /usr/local/src/ferriskey/target/release/ferriskey-operator /usr/local/bin/

EXPOSE 80

ENTRYPOINT ["ferriskey-operator"]

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

RUN pnpm run build

FROM nginx:1.28.0-alpine3.21-slim AS webapp

COPY --from=webapp-build /usr/local/src/ferriskey/dist /usr/local/src/ferriskey
COPY front/nginx.conf /etc/nginx/conf.d/default.conf
COPY front/docker-entrypoint.sh /docker-entrypoint.d/docker-entrypoint.sh

RUN chmod +x /docker-entrypoint.d/docker-entrypoint.sh
