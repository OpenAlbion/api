# --- build ---------------------------------------------------------------
FROM rust:1.85-slim AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY web ./web

# Build with cached dependencies when available (none on a fresh build).
RUN cargo build --release

# --- run ------------------------------------------------------------------
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /build/target/release/openalbion-api /usr/local/bin/openalbion-api
COPY data ./data

ENV HOST=0.0.0.0 \
    PORT=8080 \
    DATA_DIR=/app/data

EXPOSE 8080
ENTRYPOINT ["openalbion-api"]
