# syntax=docker/dockerfile:1.7

# ============================================================================
# Build stage — compiles api-gateway + seed + backoffice-api against the
# workspace. Uses cargo's release profile; the migrations directory is embedded
# into the `seed` binary at compile time via the `sqlx::migrate!` macro, so the
# runtime image doesn't ship the migrations dir or sqlx-cli.
# ============================================================================
FROM rust:1.92.0-slim-bookworm AS builder

WORKDIR /app

# Build dependencies needed for some sqlx transitive deps (pkg-config etc.)
# rustls is used (no openssl), so libssl-dev is intentionally omitted.
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY . .

# Builds release binaries for api-gateway, seed and backoffice-api in a single
# cargo invocation so the build artifact cache is shared.
RUN cargo build --release -p api-gateway -p seed -p backoffice-api

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
COPY --from=builder /app/target/release/backoffice-api /usr/local/bin/backoffice-api
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Default entrypoint runs the tenant stack (seed + api-gateway on :8000). The
# backoffice service in compose overrides the entrypoint to run backoffice-api
# (:8001) from this same image — it must NOT re-run the seed.
EXPOSE 8000 8001
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
