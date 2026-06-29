# SPEC.md

```toml
artifact_type = "spec"
id = "SPEC-CONTENT-0001"
state = "done"
owner = "platform"
created_at = "2026-06-28"
updated_at = "2026-06-28"
replaces = "none"
related_tasks = ["TASK-CONTENT-0001", "TASK-CONTENT-0002", "TASK-CONTENT-0003", "TASK-CONTENT-0004", "TASK-CONTENT-0005"]
related_decisions = []
artifacts = ["content-service/*"]
validation = ["cargo test -p content-service", "manual: crear categoria/tema/post/comentario via HTTP", "manual: verificar compresion ZSTD del body y metadata de compresion", "manual: publish/consume de eventos forum.post.created / forum.comment.created por MQTT"]
```

## Metadata

- ID: SPEC-CONTENT-0001
- Estado: `done`
- Owner: platform
- Fecha de creacion: 2026-06-28
- Ultima actualizacion: 2026-06-28
- Reemplaza: `none`
- Tareas relacionadas: `TASK-CONTENT-0001`, `TASK-CONTENT-0002`,
  `TASK-CONTENT-0003`, `TASK-CONTENT-0004`, `TASK-CONTENT-0005`
- Decisiones relacionadas: `none`

## Resumen

Crear `content-service` replicando el esqueleto hexagonal validado en
`users-service` (SPEC-USERS-0001), pero con su propio dominio: categorías,
temas (topics), posts y comentarios. A diferencia de `users-service`, este
servicio introduce dos elementos nuevos del prompt de arquitectura: la
compresión ZSTD del body de posts/comentarios, y la emisión de eventos que
disparan a `search-service` (`forum.embedding.generate.request`), aunque
ese consumo se implementará en una spec posterior dedicada a
`search-service`.

## Problema

Con `users-service` ya validamos que el patrón
hexagonal + SQLite/WAL + Inbox/Outbox + MQTT funciona end-to-end. Falta
demostrar que el mismo patrón soporta un dominio con relaciones (categoría
-> tema -> post -> comentario) y con una preocupación de almacenamiento
adicional (compresión) sin romper la separación de capas.

## Objetivo

Tener `content-service` corriendo localmente: HTTP mínimo para crear
categorías, temas, posts y comentarios; persistencia en `content.sqlite`
propia; compresión ZSTD nivel 3 del body de posts/comentarios con su
metadata; emisión de `forum.post.created` y `forum.comment.created` vía el
patrón Outbox ya validado.

## Alcance

- Incluye: entidades `Category`, `Topic`, `Post`, `Comment`; casos de uso
  de creación para cada una; compresión/descompresión ZSTD de
  `posts.body` y `comments.body`; endpoints HTTP mínimos; Inbox/Outbox +
  reutilización del worker MQTT (mismo patrón de `users-service`,
  adaptado a los nuevos topics).
- Excluye: edición/borrado de contenido, moderación, paginación avanzada,
  `search-service` (se consume su contrato pero no se implementa aquí),
  embeddings.

## Requisitos funcionales

- RF-1: crear categoría, tema, post y comentario vía HTTP, persistidos en
  `content.sqlite`.
- RF-2: `posts.body` y `comments.body` se almacenan comprimidos con ZSTD
  nivel 3, guardando `body_original_length`, `compression_algorithm`,
  `compression_level`; nunca se comprimen ids, titles, slugs ni
  timestamps.
- RF-3: crear un post o comentario genera un `outbox_events` en la misma
  transacción (`forum.post.created` / `forum.comment.created`), publicado
  después del commit por el mismo mecanismo de `publish-outbox`.
- RF-4: `GET` de un post/comentario devuelve el body descomprimido.

## Requisitos no funcionales

- RNF-1: mismos pragmas SQLite obligatorios que `users-service`
  (ver `STACK.md`), una sola conexión por proceso.
- RNF-2: el dominio no conoce ZSTD ni SQLite directamente; la
  compresión vive en el adapter de persistencia, no en las entidades.
- RNF-3: estructura de carpetas idéntica a `users-service` para mantener
  el patrón replicable a `search-service`.

## Criterios de aceptacion

- Dado un post con un body de N bytes, cuando se persiste, entonces
  `posts.body` ocupa menos espacio que el original (comprimido) y
  `body_original_length` registra N.
- Dado un post recién creado, cuando se hace `GET`, entonces el body
  devuelto es idéntico al original (round-trip de compresión correcto).
- Dado un post o comentario creado, cuando se hace commit, entonces existe
  una fila en `outbox_events` correspondiente, no publicada antes del
  commit.

## Dependencias y riesgos

- Dependencia: el mismo patrón Inbox/Outbox/MQTT de `users-service`
  (TASK-USERS-0004) se reutiliza casi sin cambios.
- Riesgo: relaciones entre categoría/tema/post/comentario podrían tentar a
  sobre-modelar. Mitigación: solo claves foráneas simples, sin reglas de
  negocio adicionales en esta iteración.
- Riesgo: elegir mal el punto de compresión (dominio vs adapter) rompe
  RNF-2. Mitigación: compresión solo en el adapter SQLite, entidades de
  dominio siempre manejan el body en texto plano.

## Plan de ejecucion

- TASK-CONTENT-0001: esqueleto del servicio + SQLite base (`content.sqlite`,
  pragmas, `init-db` con tablas `categories`, `topics`, `posts`,
  `comments`, `inbox_messages`, `outbox_events`).
- TASK-CONTENT-0002: dominio + adapter de compresión ZSTD para
  posts/comentarios (round-trip probado).
- TASK-CONTENT-0003: casos de uso `CreateCategory`, `CreateTopic`,
  `CreatePost`, `CreateComment` + repositorios.
- TASK-CONTENT-0004: transport Axum para los 4 recursos + health/ready.
- TASK-CONTENT-0005: Outbox para `forum.post.created` /
  `forum.comment.created`, reutilizando `publish-outbox`.

## Plan de validacion

- `cargo test -p content-service`.
- Walkthrough manual: crear categoría -> tema -> post -> comentario por
  HTTP, confirmar compresión y recuperación correcta del body.
- Walkthrough manual contra Mosquitto: crear post, `publish-outbox`,
  verificar `forum.post.created` recibido por un suscriptor externo.

## Trazabilidad

- Commits o PRs: pendiente (sin commit todavia en este repo)
- Archivos principales: `content-service/*`
- Resultado de validacion: `cargo test` 14/14 ok; walkthrough manual end-to-end por HTTP confirmado (categoria -> tema -> post comprimido/descomprimido -> comentario); bug real encontrado y corregido durante la implementacion: `map_insert_error` confundia violaciones de foreign key con duplicados (ya distingue por `extended_code`)
