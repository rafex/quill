# COMMANDS.md

Lista de comandos operativos del proyecto.

## Objetivo

Reducir la ambiguedad de ejecucion para agentes y humanos.

> Nota: el código Rust aún no existe en el repo. Estos comandos describen
> el contrato esperado por servicio (`users-service`, `content-service`,
> `search-service`) una vez creados como crates independientes; ajustar
> según la estructura real de workspace cuando se implemente.

### Setup

```bash
# por servicio (ejemplo content-service)
cd content-service && cargo build
```

### Desarrollo

```bash
# levantar broker MQTT local (Mosquitto)
mosquitto -c mosquitto.conf

# iniciar un servicio
cargo run -p content-service
```

### Tests

```bash
cargo test --workspace
```

### Lint y formato

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

### Build

```bash
cargo build --release --workspace
```

### Utilidad (CLI por servicio)

```bash
cargo run -p content-service -- init-db
cargo run -p content-service -- migrate
cargo run -p content-service -- vacuum
cargo run -p search-service -- rebuild-fts
cargo run -p search-service -- reindex
cargo run -p content-service -- stats
cargo run -p content-service -- publish-outbox
cargo run -p content-service -- process-inbox
```
