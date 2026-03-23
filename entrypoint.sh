#!/bin/bash
set -e

# ── Start SurrealDB in background ──
DB_USER="${DB_USER:-Actuators}"
DB_PASS="${DB_PASS:-Actuators123}"
DB_PORT="${DB_PORT:-8000}"

echo "Starting SurrealDB v2..."
surreal start \
  --user "$DB_USER" \
  --pass "$DB_PASS" \
  --bind "127.0.0.1:${DB_PORT}" \
  file:/data/surreal &

# Wait for SurrealDB to be ready
for i in $(seq 1 30); do
  if curl -sf "http://127.0.0.1:${DB_PORT}/health" > /dev/null 2>&1; then
    echo "SurrealDB ready after ${i}s"
    break
  fi
  sleep 1
done

# ── Export env vars for Actuators ──
export SURREAL_URL="ws://127.0.0.1:${DB_PORT}"
export SURREAL_USER="$DB_USER"
export SURREAL_PASS="$DB_PASS"
export SURREAL_NS="${DB_NS:-Actuators}"
export SURREAL_DB="${DB_NAME:-erp}"

# ── Start Actuators ──
echo "Starting Actuators Platform..."
exec Actuators
