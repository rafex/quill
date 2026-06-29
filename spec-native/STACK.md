# STACK.md

Fuente de verdad de la base tecnologica del proyecto.

### Runtime

- Lenguaje: Rust
- Version: estable (stable channel, sin nightly)

### Frameworks

- Axum: HTTP transport de cada microservicio. API REST mínima, cada
  endpoint invoca un único caso de uso (nunca accede al repositorio
  directamente).
- rusqlite: acceso a SQLite. Sin ORM, todo el SQL se escribe manualmente.
- rumqttc: cliente MQTT v5 para publish/subscribe entre servicios.
- serde / serde_json: serialización de comandos, eventos y DTOs.
- ONNX Runtime + tokenizers: generación de embeddings en Rust puro.

### Infraestructura

- Base de datos: SQLite por servicio (`users.sqlite`, `content.sqlite`,
  `search.sqlite`), modo WAL. Una única conexión compartida por proceso,
  protegida adecuadamente (sin pools complejos). Pragmas obligatorios:
  ```
  PRAGMA journal_mode=WAL;
  PRAGMA synchronous=NORMAL;
  PRAGMA foreign_keys=ON;
  PRAGMA busy_timeout=5000;
  PRAGMA wal_autocheckpoint=1000;
  ```
- Búsqueda: FTS5 (texto completo) + sqlite-vec (búsqueda semántica),
  combinadas en búsqueda híbrida.
- Compresión: ZSTD nivel 3 sobre `posts.body` y `comments.body`, guardando
  `body_original_length`, `compression_algorithm`, `compression_level`. No se
  comprimen ids, titles, slugs ni timestamps.
- Mensajería: Mosquitto como broker, protocolo MQTT v5.
- Hosting: Raspberry Pi, VPS pequeños, Docker, Kubernetes, PCs antiguas,
  entornos offline-first.

### Integraciones

- Mosquitto (MQTT broker): bus de eventos único entre microservicios.
  Crítico — sin él no hay comunicación inter-servicio. Topics principales:
  `forum.post.create.request`, `forum.post.created`,
  `forum.comment.create.request`, `forum.comment.created`,
  `forum.search.index.request`, `forum.embedding.generate.request`,
  `forum.embedding.generated`, `forum.deadletter`.
- Modelo de embeddings inicial: `sentence-transformers/all-MiniLM-L6-v2`
  vía ONNX Runtime. Reemplazable detrás del trait `EmbeddingProvider` sin
  tocar el resto del sistema.

### Restricciones

- Prohibido: PostgreSQL, MySQL, Redis, Elasticsearch, OpenSearch, Kafka,
  RabbitMQ, ORMs.
- Toda persistencia debe ser SQLite; todo SQL escrito manualmente.
- Generación de embeddings 100% en Rust; no usar Python en producción.
- Cada microservicio nunca accede a la base SQLite de otro servicio —
  comunicación exclusivamente por MQTT.
