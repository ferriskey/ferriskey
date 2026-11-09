FROM rust:1.99.1-trixie@sha256:09027f873d45b8ecd1dc86f1fa13b7a62b875c961f017a0e331267b350430137 AS chef

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
FROM gcr.io/distroless/cc-debian13:nonroot@sha256:08476c244888bfb425bae1ae4dd7c17f093e972aab8426883d97f3c6f280f7c1 AS runtime

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
FROM node:26.13.0-alpine3.24@sha256:8996bb5821d62a8fd9bb27c1cb07462f3fdce6e47d640c6a881ead0b064ce216 AS webapp-build

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
