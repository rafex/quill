# COMMANDS.md

Lista de comandos operativos del proyecto.

## Objetivo

Reducir la ambiguedad de ejecucion para agentes y humanos.

### Setup

```bash
cargo build --workspace
```

### Desarrollo

```bash
# levantar broker MQTT local (Mosquitto)
mosquitto -c mosquitto.conf

# iniciar un servicio (users-service / content-service / search-service)
cargo run -p content-service -- serve
```

### Tests

```bash
cargo test --workspace

# search-service tambien se puede compilar/testear con el provider ONNX real
cargo test -p search-service --no-default-features --features onnx-embeddings
```

### Lint y formato

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

### Build

```bash
cargo build --release --workspace

# build de search-service con busqueda semantica real (ONNX + MiniLM)
# en vez del stub determinista por defecto
cargo build --release -p search-service --no-default-features --features onnx-embeddings
```

### CLI por servicio

Comunes a `users-service`, `content-service` y `search-service`:

```bash
cargo run -p <servicio> -- init-db
cargo run -p <servicio> -- serve
```

`users-service` y `content-service` ademas soportan el patron Outbox/Inbox:

```bash
cargo run -p <servicio> -- publish-outbox
cargo run -p <servicio> -- process-inbox
```

`search-service` solo consume eventos (no escribe outbox propio):

```bash
cargo run -p search-service -- process-inbox
```

### EmbeddingProvider de search-service (stub vs ONNX real)

La eleccion entre el stub determinista (default, sin dependencias
nativas) y ONNX Runtime + MiniLM real es una decision de build y de
runtime, no una limitante fija (ver DEC-0006 en `DECISIONS.md`):

```bash
# 1. compilar con soporte ONNX (agrega ort/tokenizers/ureq)
cargo build --release -p search-service --no-default-features --features onnx-embeddings

# 2. descargar el modelo una sola vez (requiere red; no se hace en cada arranque)
./target/release/search-service download-model

# 3. arrancar pidiendo el provider real
SEARCH_EMBEDDING_PROVIDER=onnx \
SEARCH_ONNX_MODEL_DIR=models/all-MiniLM-L6-v2 \
./target/release/search-service serve
```

Variables de entorno relevantes: `SEARCH_DB_PATH`, `SEARCH_HTTP_ADDR`,
`SEARCH_EMBEDDING_PROVIDER` (`stub` por defecto, o `onnx`),
`SEARCH_ONNX_MODEL_DIR` (default `models/all-MiniLM-L6-v2`),
`MQTT_HOST`, `MQTT_PORT`.

### Utilidad pendiente

Estos comandos estan en el diseno original pero aun no implementados:

```bash
cargo run -p content-service -- migrate
cargo run -p content-service -- vacuum
cargo run -p search-service -- rebuild-fts
cargo run -p search-service -- reindex
cargo run -p content-service -- stats
```
