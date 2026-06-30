#!/usr/bin/env bash
set -eu

BIN=/usr/local/bin/search-service

echo "[search-service] initializing database..."
"$BIN" init-db

echo "[search-service] starting inbox worker (provider: ${SEARCH_EMBEDDING_PROVIDER:-stub})..."
"$BIN" process-inbox &

echo "[search-service] starting HTTP server..."
exec "$BIN" serve
