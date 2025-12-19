# syntax=docker/dockerfile:1.6

## Build stage: compile all binaries we need (api, engine, ws, db_filler)
FROM rust:1.77-slim-bookworm AS builder

# System deps for sqlx/Postgres + OpenSSL
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        libpq-dev \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Pre-cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY api/Cargo.toml api/Cargo.toml
COPY engine/Cargo.toml engine/Cargo.toml
COPY ws/Cargo.toml ws/Cargo.toml
COPY db_filler/Cargo.toml db_filler/Cargo.toml
COPY redis/Cargo.toml redis/Cargo.toml
COPY shared/Cargo.toml shared/Cargo.toml
COPY db/Cargo.toml db/Cargo.toml

RUN mkdir -p api/src engine/src ws/src db_filler/src redis/src shared/src db/src \
    && touch api/src/lib.rs engine/src/lib.rs ws/src/lib.rs db_filler/src/lib.rs redis/src/lib.rs shared/src/lib.rs db/src/lib.rs

RUN cargo fetch --locked

# Now copy the full workspace
COPY . .

# Build release binaries (adjust packages if you add more entrypoints)
RUN cargo build --release --locked \
    -p api -p engine -p ws -p db_filler

## Runtime stage
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
        libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binaries
COPY --from=builder /app/target/release/api /usr/local/bin/api
COPY --from=builder /app/target/release/engine /usr/local/bin/engine
COPY --from=builder /app/target/release/ws /usr/local/bin/ws
COPY --from=builder /app/target/release/db_filler /usr/local/bin/db_filler

# Default runtime env
ENV REDIS_URL=redis://redis:6379/ \
    DATABASE_URL=postgres://postgres:postgres@postgres:5432/cex \
    API_BIND=0.0.0.0:8080 \
    WS_BIND=0.0.0.0:9000 \
    APP_BIN=api

# Simple entrypoint that picks which binary to run
RUN printf '#!/bin/sh\nset -e\nBIN="${APP_BIN:-api}"\necho "Starting ${BIN}"\nexec "/usr/local/bin/${BIN}"\n' > /entrypoint.sh \
    && chmod +x /entrypoint.sh

EXPOSE 8080 9000

ENTRYPOINT ["/entrypoint.sh"]
