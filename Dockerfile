ARG WOLFI_BASE=cgr.dev/chainguard/wolfi-base@sha256:c9a27ee8d2d441f941de2f8e4c2c8ddb0b313adb5d14ab934b19f467b9ea8083

FROM ${WOLFI_BASE} AS rust-build

WORKDIR /usr/local/src/ferriskey

ENV CARGO_HOME=/usr/local/cargo

# hadolint ignore=DL3018
RUN set -eux ;\
  apk update --no-cache && apk upgrade --no-cache ;\
  apk add --no-cache rust build-base pkgconf openssl-dev curl ;\
  cargo install --root /usr/local/cargo sqlx-cli --no-default-features --features postgres

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

FROM ${WOLFI_BASE} AS runtime

RUN set -eux ;\
  apk update --no-cache && apk upgrade --no-cache ;\
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

FROM ${WOLFI_BASE} AS webapp-build

WORKDIR /usr/local/src/ferriskey

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

# hadolint ignore=DL3018
RUN set -eux ;\
  apk update --no-cache && apk upgrade --no-cache ;\
  apk add --no-cache nodejs-24 corepack ;\
  corepack enable ;\
  corepack prepare pnpm@10.30.0 --activate

COPY front/package.json front/pnpm-lock.yaml ./

RUN pnpm install --frozen-lockfile

COPY front/ .

RUN pnpm run build

FROM docker.angie.software/angie:1.11.3-minimal AS webapp

COPY --from=webapp-build /usr/local/src/ferriskey/dist /usr/local/src/ferriskey
COPY front/default.conf /etc/angie/http.d/default.conf
COPY --chmod=0755 front/docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["angie", "-g", "daemon off;"]
