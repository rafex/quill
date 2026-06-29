# TASKS.md

```toml
artifact_type = "task_file"
initiative = "users-service-bootstrap"
spec_id = "SPEC-USERS-0001"
owner = "platform"
state = "done"
```

## Metadata

- Iniciativa: users-service-bootstrap
- Spec relacionada: SPEC-USERS-0001
- Owner: platform
- Estado general: `done`

## Tareas

### TASK-USERS-0001 - Esqueleto del servicio y SQLite base

```toml
id = "TASK-USERS-0001"
title = "Esqueleto del servicio y SQLite base"
state = "done"
owner = "platform"
dependencies = []
expected_files = ["users-service/Cargo.toml", "users-service/src/infrastructure/db.rs", "users-service/src/main.rs"]
close_criteria = "cargo run -p users-service ejecuta init-db y crea users.sqlite con WAL y las tablas base"
validation = ["test de integracion: pragmas aplicados", "walkthrough manual de init-db"]
```

Crea el crate `users-service` con las carpetas
`transport/application/domain/ports/adapters/infrastructure`, el módulo de
conexión SQLite (una sola conexión por proceso) con los pragmas
obligatorios de `STACK.md`, y el comando CLI `init-db` que crea el schema
inicial (`users`, `inbox_messages`, `outbox_events`).

### TASK-USERS-0002 - Dominio User y caso de uso CreateUser

```toml
id = "TASK-USERS-0002"
title = "Dominio User y caso de uso CreateUser"
state = "done"
owner = "platform"
dependencies = ["TASK-USERS-0001"]
expected_files = ["users-service/src/domain/user.rs", "users-service/src/ports/user_repository.rs", "users-service/src/adapters/sqlite_user_repository.rs", "users-service/src/application/create_user.rs"]
close_criteria = "Existe entidad User, port UserRepository, adapter SqliteUserRepository y caso de uso CreateUser con test de integracion contra SQLite real"
validation = ["tests unitarios de dominio", "test de integracion sobre SqliteUserRepository"]
```

Define la entidad `User` (value objects mínimos: email, username), el
port `UserRepository` y su implementación `SqliteUserRepository`, y el
caso de uso `CreateUser` (más `GetUserById` para soportar la siguiente
tarea). El dominio no debe importar nada de `adapters` ni
`infrastructure`.

### TASK-USERS-0003 - Transport HTTP (Axum)

```toml
id = "TASK-USERS-0003"
title = "Transport HTTP con Axum"
state = "done"
owner = "platform"
dependencies = ["TASK-USERS-0002"]
expected_files = ["users-service/src/transport/http.rs", "users-service/src/transport/health.rs"]
close_criteria = "POST /users y GET /users/:id funcionan end-to-end; /health y /ready responden"
validation = ["test de integracion HTTP con servidor de prueba", "walkthrough manual con curl"]
```

Expone `POST /users` y `GET /users/:id` invocando únicamente los casos de
uso de TASK-USERS-0002 (nunca el repositorio directamente), más
`/health` y `/ready`.

### TASK-USERS-0004 - Inbox/Outbox y worker MQTT minimo

```toml
id = "TASK-USERS-0004"
title = "Inbox/Outbox y worker MQTT minimo"
state = "done"
owner = "platform"
dependencies = ["TASK-USERS-0002"]
expected_files = ["users-service/src/infrastructure/mqtt.rs", "users-service/src/infrastructure/outbox_publisher.rs", "users-service/src/infrastructure/inbox_worker.rs"]
close_criteria = "CreateUser escribe un outbox_event en la misma transaccion; un publisher independiente lo envia por MQTT tras el commit; un mensaje de prueba duplicado entrante no se reprocesa"
validation = ["test de integracion: outbox no se publica antes del commit", "test de idempotencia con mensaje duplicado en inbox_messages", "walkthrough manual contra Mosquitto local"]
```

Implementa el patrón Outbox (escribir evento + commit, publicar después) y
el patrón Inbox (guardar mensaje entrante antes de procesar) usando
`rumqttc`. Sirve como plantilla para `content-service` y `search-service`.
