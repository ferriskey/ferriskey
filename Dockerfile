FROM rust:1.89-bookworm AS rust-build

WORKDIR /usr/local/src/ferriskey

RUN cargo install sqlx-cli --no-default-features --features postgres

COPY Cargo.toml Cargo.lock ./
COPY api/Cargo.toml ./api/
COPY core/Cargo.toml ./core/
COPY operator/Cargo.toml ./operator/

RUN \
    mkdir -p api/src core/src entity/src operator/src && \
    echo "fn main() {}" > api/src/main.rs && \
    touch core/src/lib.rs && \
    echo "fn main() {}" > operator/src/main.rs && \
    cargo build --release

COPY api api
COPY core core
COPY operator operator

RUN \
    touch api/src/main.rs && \
    touch core/src/lib.rs && \
    touch operator/src/main.rs && \
    cargo build --release

FROM debian:bookworm-slim AS runtime

RUN \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates=20230311 \
    libssl3=3.0.16-1~deb12u1 && \
    rm -rf /var/lib/apt/lists/* && \
    adduser --system --group --no-create-home --disabled-login ferriskey

USER ferriskey

FROM runtime AS api

COPY --from=rust-build /usr/local/src/ferriskey/target/release/ferriskey-server /usr/local/bin/
COPY --from=rust-build /usr/local/src/ferriskey/core/migrations /usr/local/bin/ferriskey-migrations
COPY --from=rust-build /usr/local/cargo/bin/sqlx /usr/local/bin/

EXPOSE 80

ENTRYPOINT ["ferriskey-server"]

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

COPY --from=webapp-build /usr/local/src/ferriskey/dist /usr/share/nginx/html
COPY front/nginx.conf /etc/nginx/conf.d/default.conf
COPY front/env.sh /docker-entrypoint.d/env.sh

RUN chmod +x /docker-entrypoint.d/env.sh
