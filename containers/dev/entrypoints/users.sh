#!/usr/bin/env bash
set -eu

BIN=/usr/local/bin/users-service

echo "[users-service] initializing database..."
"$BIN" init-db

echo "[users-service] starting inbox worker..."
"$BIN" process-inbox &

echo "[users-service] starting outbox loop (every 2s)..."
(while true; do "$BIN" publish-outbox 2>/dev/null || true; sleep 2; done) &

echo "[users-service] starting HTTP server..."
exec "$BIN" serve
