# syntax=docker/dockerfile:1

# Pin versi untuk build reproducible
ARG RUST_VERSION=1.95
ARG DEBIAN_VERSION=bookworm

# -----------------------------------------------------------------------------
# Stage 1: Generate dependency recipe (api crate)
# -----------------------------------------------------------------------------
FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} AS chef
WORKDIR /app

RUN cargo install cargo-chef --locked

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo chef prepare --recipe-path recipe.json

# -----------------------------------------------------------------------------
# Stage 2: Build release binaries (api + migration)
# -----------------------------------------------------------------------------
FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} AS builder
WORKDIR /app

RUN cargo install cargo-chef --locked

COPY --from=chef /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Hanya binary yang dibutuhkan runtime (bukan generate-paseto-keys)
RUN cargo build --release --bin api && \
    CARGO_TARGET_DIR=/app/target cargo build --release \
      --manifest-path src/infrastructure/database/migrations/Cargo.toml

# -----------------------------------------------------------------------------
# Stage 3: Runtime API
# -----------------------------------------------------------------------------
FROM debian:${DEBIAN_VERSION}-slim AS api

ARG DEBIAN_VERSION=bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin appuser

COPY --from=builder /app/target/release/api /usr/local/bin/api

USER appuser
WORKDIR /app

EXPOSE 8080
# Env di-inject oleh docker compose / orchestrator (bukan .env di image)
ENV APP_SERVER__HOST=0.0.0.0 \
    APP_SERVER__PORT=8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=40s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8080/ >/dev/null || exit 1

ENTRYPOINT ["/usr/local/bin/api"]

# -----------------------------------------------------------------------------
# Stage 4: Runtime migrator (one-shot di compose)
# -----------------------------------------------------------------------------
FROM debian:${DEBIAN_VERSION}-slim AS migrate

ARG DEBIAN_VERSION=bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10002 --no-create-home --shell /usr/sbin/nologin migrator

COPY --from=builder /app/target/release/migration /usr/local/bin/migration

USER migrator
WORKDIR /app

ENTRYPOINT ["/usr/local/bin/migration"]
CMD ["up"]