# Justfile — operational tasks (run, DB, MQTT, reindex, containers).
# For build tasks (compile, test, lint) use: make <target>
#
# Requires: just (https://github.com/casey/just)
# Optional: .env file for MQTT_HOST, MQTT_PORT, *_DB_PATH, *_HTTP_ADDR

set dotenv-load := true
set shell := ["bash", "-cu"]

# Prefer podman-compose; fall back to docker compose
COMPOSE := `command -v podman-compose 2>/dev/null || echo "docker compose"`
COMPOSE_DEV := COMPOSE + " -f containers/dev/compose.yml"

import 'scripts/db.just'
import 'scripts/mqtt.just'

# Show all available recipes
[private]
default:
    @just --list --unsorted

# ── Frontend ──────────────────────────────────────────────────────────────────

# Start Vite dev server for the web frontend (TypeScript + Wasm)
[group('web')]
dev-web:
    npm --prefix web run dev

# Build the frontend for production (TypeScript check + Vite bundle)
[group('web')]
build-web:
    make build-web

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
    @echo "web:"; curl -sf -o /dev/null -w "%{http_code}\n" http://localhost:8090/ || echo "OFFLINE"

# Run a hybrid search query
# Usage: just search "cars and driving"
[group('smoke')]
search q:
    curl -sf "http://localhost:8082/search?q={{q}}&limit=5" | jq .

# ── Containers (dev) ──────────────────────────────────────────────────────────

# Build all container images for the dev environment
[group('containers')]
dev-build:
    {{COMPOSE_DEV}} build

# Start the full dev stack (Mosquitto + 3 services) in the foreground
[group('containers')]
dev-up:
    {{COMPOSE_DEV}} up

# Start the full dev stack in the background
[group('containers')]
dev-up-detach:
    {{COMPOSE_DEV}} up -d

# Stop and remove containers (keeps named volumes / data)
[group('containers')]
dev-down:
    {{COMPOSE_DEV}} down

# Stop containers AND remove all data volumes (full reset)
[group('containers')]
dev-reset:
    {{COMPOSE_DEV}} down -v

# Stream logs from all services
[group('containers')]
dev-logs:
    {{COMPOSE_DEV}} logs -f

# Stream logs from a single service
# Usage: just dev-logs-svc content-service
[group('containers')]
dev-logs-svc svc:
    {{COMPOSE_DEV}} logs -f {{svc}}

# Show running container status
[group('containers')]
dev-ps:
    {{COMPOSE_DEV}} ps

# Rebuild and restart a single service without touching the others
# Usage: just dev-restart content-service
[group('containers')]
dev-restart svc:
    {{COMPOSE_DEV}} up -d --build {{svc}}

# ── Remote access (Podman runs on a bastion host) ───────────────────────────────

# Open an SSH tunnel forwarding the app ports (8080-8082, 8090) from bastion
# to localhost, so you can reach the stack from a browser on this machine.
# Runs in the foreground — Ctrl-C to close the tunnel.
[group('containers')]
tunnel:
    ssh -N \
        -L 8080:localhost:8080 \
        -L 8081:localhost:8081 \
        -L 8082:localhost:8082 \
        -L 8090:localhost:8090 \
        bastion

# Same as `tunnel`, but detached (runs in the background).
# Usage: just tunnel-up   /   just tunnel-down to close it
[group('containers')]
tunnel-up:
    ssh -fN \
        -o ExitOnForwardFailure=yes \
        -L 8080:localhost:8080 \
        -L 8081:localhost:8081 \
        -L 8082:localhost:8082 \
        -L 8090:localhost:8090 \
        bastion
    @echo "tunnel up — web: http://localhost:8090  users: 8080  content: 8081  search: 8082"

# Close the background tunnel opened by `tunnel-up`
[group('containers')]
tunnel-down:
    pkill -f "ssh -fN .*-L 8090:localhost:8090.*bastion" || echo "no tunnel running"
