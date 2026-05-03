#!/usr/bin/env sh
# Container boot order:
#   1. `seed` runs sqlx migrations (idempotent — tracks _sqlx_migrations) +
#      seeds permissions/roles/store/admin/products/etc. The seed body uses
#      ON CONFLICT DO NOTHING throughout, so re-running on an already-seeded
#      database is a no-op.
#   2. `api-gateway` takes over PID 1 via exec.
set -e

echo "[entrypoint] running migrations + seed..."
seed
echo "[entrypoint] starting api-gateway on :8000..."
exec api-gateway
