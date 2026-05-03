# syntax=docker/dockerfile:1.7

# ============================================================================
# Build stage — compiles api-gateway + seed against the workspace.
# Uses cargo's release profile; the migrations directory is embedded into the
# `seed` binary at compile time via the `sqlx::migrate!` macro, so the runtime
# image doesn't ship the migrations dir or sqlx-cli.
# ============================================================================
FROM rust:1.92.0-slim-bookworm AS builder

WORKDIR /app

# Build dependencies needed for some sqlx transitive deps (pkg-config etc.)
# rustls is used (no openssl), so libssl-dev is intentionally omitted.
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY . .

# Builds release binaries for both api-gateway and seed in a single cargo
# invocation so the build artifact cache is shared.
RUN cargo build --release -p api-gateway -p seed

# ============================================================================
# Runtime stage — slim debian with only what the binaries need at runtime:
# CA certificates for outbound TLS (e.g. payment/notification adapters in v1.1).
# ============================================================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/api-gateway /usr/local/bin/api-gateway
COPY --from=builder /app/target/release/seed /usr/local/bin/seed
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

EXPOSE 8000
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
