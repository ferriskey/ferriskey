FROM rust:1.86-bookworm AS rust-build
WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY api/Cargo.toml api/
COPY core/Cargo.toml core/
COPY core/entity core/entity
COPY operator/Cargo.toml operator/

RUN mkdir -p api/src/lib api/src/bin core/src operator/src \
 && echo "fn main() {}" > api/src/bin/main.rs \
 && echo "" > api/src/lib.rs \
 && echo "pub mod lib;" > api/src/lib/mod.rs \
 && echo "" > api/src/lib/lib.rs \
 && echo "" > core/src/lib.rs \
 && echo "" > operator/src/lib.rs

RUN cargo install sqlx-cli --no-default-features --features postgres
RUN cargo build --release --bin ferriskey-server && \
    rm -rf api/src core/src operator/src


# Copy actual source code
COPY api api
COPY core core
COPY operator operator

RUN cargo build --release --bin ferriskey-server && \
    strip target/release/ferriskey-server


FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN adduser --system --group --no-create-home --disabled-login ferriskey && \
    chown -R ferriskey:ferriskey /app

# Copy binaries and migrations
COPY --from=rust-build --chown=ferriskey:ferriskey /app/target/release/ferriskey-server /usr/local/bin/
COPY --from=rust-build --chown=ferriskey:ferriskey /usr/local/cargo/bin/sqlx /usr/local/bin/
COPY --from=rust-build --chown=ferriskey:ferriskey /app/core/migrations /app/migrations

USER ferriskey

EXPOSE 80

CMD [ "/usr/local/bin/ferriskey-server" ]