# DB recipes — imported by root Justfile
# Env vars (override via .env or shell):
#   USERS_DB_PATH    (default: users.sqlite)
#   CONTENT_DB_PATH  (default: content.sqlite)
#   SEARCH_DB_PATH   (default: search.sqlite)

# Initialize all three databases
[group('db')]
init-all:
    just init-users
    just init-content
    just init-search

# Initialize users.sqlite
[group('db')]
init-users:
    cargo run -p users-service -- init-db

# Initialize content.sqlite
[group('db')]
init-content:
    cargo run -p content-service -- init-db

# Initialize search.sqlite
[group('db')]
init-search:
    cargo run -p search-service -- init-db

# Print row counts for all services
[group('db')]
stats-all:
    @echo "=== users-service ==="
    cargo run -p users-service -- stats
    @echo ""
    @echo "=== content-service ==="
    cargo run -p content-service -- stats
    @echo ""
    @echo "=== search-service ==="
    cargo run -p search-service -- stats

# Print stats for users-service only
[group('db')]
stats-users:
    cargo run -p users-service -- stats

# Print stats for content-service only
[group('db')]
stats-content:
    cargo run -p content-service -- stats

# Print stats for search-service only
[group('db')]
stats-search:
    cargo run -p search-service -- stats

# VACUUM all databases (reclaim disk space after bulk deletes)
[group('db')]
vacuum-all:
    cargo run -p users-service -- vacuum
    cargo run -p content-service -- vacuum
    cargo run -p search-service -- vacuum

# VACUUM users.sqlite
[group('db')]
vacuum-users:
    cargo run -p users-service -- vacuum

# VACUUM content.sqlite
[group('db')]
vacuum-content:
    cargo run -p content-service -- vacuum

# VACUUM search.sqlite
[group('db')]
vacuum-search:
    cargo run -p search-service -- vacuum

# Flush pending outbox events for users-service
[group('db')]
outbox-users:
    cargo run -p users-service -- publish-outbox

# Flush pending outbox events for content-service
[group('db')]
outbox-content:
    cargo run -p content-service -- publish-outbox

# Republish all content and flush outbox (full search reindex in one step)
[group('db')]
reindex:
    cargo run -p content-service -- reindex

# Download the ONNX MiniLM model (requires --features onnx-embeddings build)
[group('db')]
download-model:
    cargo run -p search-service --features onnx-embeddings -- download-model
