# TASKS.md

```toml
artifact_type = "task_file"
initiative = "content-service-bootstrap"
spec_id = "SPEC-CONTENT-0001"
owner = "platform"
state = "done"
```

## Metadata

- Iniciativa: content-service-bootstrap
- Spec relacionada: SPEC-CONTENT-0001
- Owner: platform
- Estado general: `done`

## Tareas

### TASK-CONTENT-0001 - Esqueleto del servicio y SQLite base

```toml
id = "TASK-CONTENT-0001"
title = "Esqueleto del servicio y SQLite base"
state = "done"
owner = "platform"
dependencies = []
expected_files = ["content-service/Cargo.toml", "content-service/src/infrastructure/db.rs", "content-service/src/main.rs"]
close_criteria = "cargo run -p content-service init-db crea content.sqlite con WAL y las tablas categories/topics/posts/comments/inbox_messages/outbox_events"
validation = ["test de integracion: pragmas aplicados", "walkthrough manual de init-db"]
```

Crea el crate `content-service` con la misma estructura de capas que
`users-service` (transport/application/domain/ports/adapters/infrastructure).

### TASK-CONTENT-0002 - Compresion ZSTD de body

```toml
id = "TASK-CONTENT-0002"
title = "Compresion ZSTD de body (posts y comentarios)"
state = "done"
owner = "platform"
dependencies = ["TASK-CONTENT-0001"]
expected_files = ["content-service/src/infrastructure/compression.rs"]
close_criteria = "round-trip compress/decompress nivel 3 preserva el body original y registra body_original_length"
validation = ["test unitario de round-trip", "test con body vacio y body grande"]
```

Implementa compresión/descompresión ZSTD nivel 3 en `infrastructure`
(nunca en `domain`), reutilizable por los adapters de `posts` y
`comments`.

### TASK-CONTENT-0003 - Dominio y casos de uso de creacion

```toml
id = "TASK-CONTENT-0003"
title = "Dominio y casos de uso de creacion"
state = "done"
owner = "platform"
dependencies = ["TASK-CONTENT-0002"]
expected_files = ["content-service/src/domain/*.rs", "content-service/src/ports/*.rs", "content-service/src/adapters/*.rs", "content-service/src/application/*.rs"]
close_criteria = "CreateCategory, CreateTopic, CreatePost, CreateComment funcionan con test de integracion contra SQLite real, incluyendo compresion en posts/comments"
validation = ["tests unitarios de dominio", "tests de integracion de cada repositorio"]
```

Entidades `Category`, `Topic`, `Post`, `Comment` y sus repositorios
SQLite. El body de post/comment se comprime en el adapter antes de
persistir y se descomprime al leer.

### TASK-CONTENT-0004 - Transport HTTP

```toml
id = "TASK-CONTENT-0004"
title = "Transport HTTP con Axum"
state = "done"
owner = "platform"
dependencies = ["TASK-CONTENT-0003"]
expected_files = ["content-service/src/transport/http.rs", "content-service/src/transport/health.rs"]
close_criteria = "Endpoints de creacion y lectura para los 4 recursos responden end-to-end; /health y /ready disponibles"
validation = ["walkthrough manual con curl: categoria -> tema -> post -> comentario"]
```

### TASK-CONTENT-0005 - Outbox de eventos de contenido

```toml
id = "TASK-CONTENT-0005"
title = "Outbox para forum.post.created y forum.comment.created"
state = "done"
owner = "platform"
dependencies = ["TASK-CONTENT-0003"]
expected_files = ["content-service/src/adapters/sqlite_post_repository.rs", "content-service/src/adapters/sqlite_comment_repository.rs"]
close_criteria = "Crear un post o comentario escribe un outbox_event en la misma transaccion; publish-outbox lo envia por MQTT tras el commit"
validation = ["test de integracion: outbox no se publica antes del commit", "walkthrough manual contra Mosquitto local"]
```

Reutiliza el mecanismo de `publish-outbox` ya construido para
`users-service`, adaptado a los nuevos topics de contenido.

### TASK-CONTENT-0006 - Creacion de posts/comentarios via MQTT

```toml
id = "TASK-CONTENT-0006"
title = "Wire forum.post.create.request y forum.comment.create.request"
state = "done"
owner = "platform"
dependencies = ["TASK-CONTENT-0005"]
expected_files = ["content-service/src/main.rs"]
close_criteria = "Un mensaje en forum.post.create.request o forum.comment.create.request crea el recurso real (no un placeholder) e idempotencia via request_id"
validation = ["walkthrough manual contra Mosquitto real: crear post via MQTT sin pasar por HTTP, confirmar forum.post.created emitido tras publish-outbox", "duplicar request_id y confirmar que no se reprocesa"]
```

Reemplaza el placeholder `forum.content.command` (que no llamaba a ningun
caso de uso real) por dos suscripciones explicitas que invocan
`CreatePost`/`CreateComment`, los mismos casos de uso que ya usa el
transport HTTP. La idempotencia usa `request_id` del payload como
`message_id` del Inbox. `forum.post.created`/`forum.comment.created` se
siguen publicando solos via el Outbox ya existente — no hubo que tocar
los repositorios.

### TASK-CONTENT-0007 - forum.search.reindex.request

```toml
id = "TASK-CONTENT-0007"
title = "Reemitir outbox events de todo el contenido existente"
state = "done"
owner = "platform"
dependencies = ["TASK-CONTENT-0006"]
expected_files = ["content-service/src/ports/post_repository.rs", "content-service/src/ports/comment_repository.rs", "content-service/src/adapters/sqlite_post_repository.rs", "content-service/src/adapters/sqlite_comment_repository.rs", "content-service/src/application/reindex_content.rs", "content-service/src/main.rs"]
close_criteria = "forum.search.reindex.request genera un outbox_event fresco (event_id propio) por cada post/comentario existente, sin tocar las tablas posts/comments"
validation = ["tests de integracion: republish_all crea event_id distintos sin duplicar filas en posts/comments", "walkthrough manual end-to-end: indexar un post, pedir reindex, publicar y consumir de nuevo, confirmar que search-service no duplica filas (ver DEC-0008)"]
```

Agrega `republish_all` al port `PostRepository`/`CommentRepository`
(vive en el adapter, los casos de uso siguen sin tocar SQLite
directamente) y el caso de uso `ReindexContent` que lo orquesta para
ambos repositorios. Requirió `DEC-0008` (event_id por emision +
upsert por ext_id en `search-service`) para que el reindex realmente
reprocese en vez de ser descartado como duplicado.
