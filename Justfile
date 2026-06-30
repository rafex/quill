# Justfile — operational tasks (run, DB, MQTT, reindex).
# For build tasks (compile, test, lint) use: make <target>
#
# Requires: just (https://github.com/casey/just)
# Optional: .env file for MQTT_HOST, MQTT_PORT, *_DB_PATH, *_HTTP_ADDR

set dotenv-load := true
set shell := ["bash", "-cu"]

import 'scripts/db.just'
import 'scripts/mqtt.just'

# Show all available recipes
[private]
default:
    @just --list --unsorted

# ── Services ──────────────────────────────────────────────────────────────────

# Run users-service HTTP server (USERS_HTTP_ADDR, default 0.0.0.0:8080)
[group('serve')]
serve-users:
    cargo run -p users-service -- serve

# Run content-service HTTP server (CONTENT_HTTP_ADDR, default 0.0.0.0:8081)
[group('serve')]
serve-content:
    cargo run -p content-service -- serve

# Run search-service HTTP server (SEARCH_HTTP_ADDR, default 0.0.0.0:8082)
[group('serve')]
serve-search:
    cargo run -p search-service -- serve

# Run search-service with real ONNX/MiniLM embeddings
[group('serve')]
serve-search-onnx:
    cargo run -p search-service --features onnx-embeddings -- serve

# ── Inbox workers ─────────────────────────────────────────────────────────────

# Start users-service inbox worker (listens on forum.user.command)
[group('inbox')]
inbox-users:
    cargo run -p users-service -- process-inbox

# Start content-service inbox worker (post/comment create + reindex)
[group('inbox')]
inbox-content:
    cargo run -p content-service -- process-inbox

# Start search-service inbox worker (indexes content + embedding requests)
[group('inbox')]
inbox-search:
    cargo run -p search-service -- process-inbox

# Start search-service inbox worker with ONNX embeddings
[group('inbox')]
inbox-search-onnx:
    cargo run -p search-service --features onnx-embeddings -- process-inbox

# ── HTTP smoke tests (requires running service) ───────────────────────────────

# Health-check all three services
[group('smoke')]
health:
    @echo "users-service:"; curl -sf http://localhost:8080/health | jq . || echo "OFFLINE"
    @echo "content-service:"; curl -sf http://localhost:8081/health | jq . || echo "OFFLINE"
    @echo "search-service:"; curl -sf http://localhost:8082/health | jq . || echo "OFFLINE"

# Run a hybrid search query
# Usage: just search "cars and driving"
[group('smoke')]
search q:
    curl -sf "http://localhost:8082/search?q={{q}}&limit=5" | jq .
