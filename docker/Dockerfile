FROM rust:1.86-bookworm AS rust-build
WORKDIR /app

COPY Cargo.toml Cargo.lock ./





# Copy actual source code
COPY api api
COPY core core
COPY operator operator

RUN cargo install sqlx-cli --no-default-features --features postgres
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