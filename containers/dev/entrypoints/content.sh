#!/usr/bin/env bash
set -eu

BIN=/usr/local/bin/content-service

echo "[content-service] initializing database..."
"$BIN" init-db

echo "[content-service] starting inbox worker..."
"$BIN" process-inbox &

echo "[content-service] starting outbox loop (every 2s)..."
(while true; do "$BIN" publish-outbox 2>/dev/null || true; sleep 2; done) &

echo "[content-service] starting HTTP server..."
exec "$BIN" serve
