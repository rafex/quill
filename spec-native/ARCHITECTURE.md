# ARCHITECTURE.md

Describe la arquitectura actual del proyecto.

### Vision general

Plataforma de microservicios ligeros orientados a eventos. Cada
microservicio sigue arquitectura hexagonal (Ports & Adapters) + DDD, es
dueño absoluto de su propia base SQLite, y se comunica con los demás
exclusivamente vía MQTT (nunca por acceso directo a bases de datos ajenas
ni llamadas síncronas entre servicios). La dependencia de código siempre
apunta hacia el dominio; nunca se permiten dependencias inversas. Cada
servicio implementa Inbox (para consumo idempotente de mensajes) y Outbox
(para publicar eventos solo después del commit local).

### Modulos principales

- `users-service` (`users.sqlite`): usuarios, autenticación, perfiles.
- `content-service` (`content.sqlite`): categorías, temas, posts,
  comentarios. Comprime `posts.body` / `comments.body` con ZSTD.
- `search-service` (`search.sqlite`): índice FTS5, embeddings (sqlite-vec),
  búsqueda híbrida.

Cada servicio se estructura internamente en capas:
```
transport      → Axum handlers, solo invocan casos de uso
application     → casos de uso / application services
domain          → entities, value objects, domain services
ports           → interfaces (repositorios, EmbeddingProvider, etc.)
adapters        → implementaciones de ports (ej. SQLiteRepository)
infrastructure  → SQLite, MQTT (rumqttc), ONNX Runtime
```

### Flujo principal

Creación de un post:
1. `content-service` recibe `forum.post.create.request` (o HTTP) y lo
   guarda en `inbox_messages` antes de procesar (idempotencia).
2. El caso de uso valida y persiste el post (con compresión ZSTD del body)
   y escribe un evento en `outbox_events` en la misma transacción.
3. Commit. Un publisher independiente lee `outbox_events` y publica
   `forum.post.created` y `forum.embedding.generate.request` por MQTT.
4. `search-service` consume el mensaje, genera el embedding (ONNX +
   MiniLM), lo guarda en sqlite-vec, indexa en FTS5, y publica
   `forum.embedding.generated`.
5. Una búsqueda combina: embedding de la query → sqlite-vec + FTS5 →
   `score = 0.60*vector + 0.40*bm25` → resultados con id, tipo, title,
   snippet, score.

### Restricciones

- Dependencias prohibidas: PostgreSQL, MySQL, Redis, Elasticsearch,
  OpenSearch, Kafka, RabbitMQ, ORMs.
- Ningún microservicio accede a la SQLite de otro; solo MQTT.
- Los Use Cases nunca acceden a SQLite directamente, solo vía repositorios
  (interfaces) implementados por `SQLiteRepository`.
- Los handlers HTTP nunca acceden al repositorio directamente, solo a
  casos de uso.
- Nunca publicar un evento MQTT antes del commit local (outbox pattern).

### Riesgos

- Concurrencia de escritura en SQLite con una sola conexión por proceso:
  mitigado con WAL + `busy_timeout=5000` + diseño de acceso secuencial.
- Pérdida de eventos MQTT (broker caído): mitigado con outbox persistente
  y reintentos del publisher; mensajes fallidos van a `forum.deadletter`.
- Procesamiento duplicado de mensajes: mitigado con `inbox_messages` e
  idempotencia explícita en cada caso de uso.
- Cambio de modelo de embeddings: aislado detrás del trait
  `EmbeddingProvider` para no romper el resto del sistema.
