# PRODUCT.md

Fuente de verdad del producto.

### Problema

Construir una plataforma de microservicios ligeros orientados a eventos
(Lightweight Event-Driven Microservices), inicialmente aplicada a un foro,
donde cada servicio sea dueño absoluto de su propia base SQLite y la
comunicación entre servicios ocurra exclusivamente vía MQTT. El objetivo no
es "un foro" sino una arquitectura de referencia portable que funcione igual
en una Raspberry Pi, un VPS pequeño, Docker, Kubernetes o un entorno
offline-first.

### Usuarios

- Operadores de comunidades pequeñas/medianas que quieren auto-hospedar un
  foro con búsqueda híbrida (semántica + texto) sin pagar infraestructura
  pesada (sin Postgres, Redis, Elasticsearch, Kafka).
- Arquitectos/desarrolladores que necesitan una referencia real de
  arquitectura hexagonal + DDD + event-driven en Rust, con bajo consumo de
  RAM/CPU, totalmente portable y desplegable de forma independiente por
  servicio.

### Objetivos

- Cada microservicio (`users-service`, `content-service`, `search-service`)
  es autónomo: su propio `*.sqlite`, su propio despliegue, sin acceder nunca
  a la base de datos de otro servicio.
- Búsqueda híbrida funcional: embedding (sqlite-vec) + FTS5, combinados con
  `score = 0.60 * vector + 0.40 * bm25`.
- Comunicación inter-servicio 100% MQTT, con patrones Inbox/Outbox para
  garantizar idempotencia y que nunca se publique antes del commit.
- Footprint mínimo: debe correr cómodo en Raspberry Pi o un VPS pequeño.
- Generación de embeddings 100% en Rust (ONNX Runtime + tokenizers), sin
  dependencia de Python.

### No objetivos

- No es un foro "feature-complete" (sin moderación avanzada, gamificación,
  etc.) en esta fase inicial.
- No se usará base de datos central compartida entre servicios.
- No se usará ORM; todo el SQL se escribe a mano.
- No se usarán PostgreSQL, MySQL, Redis, Elasticsearch, OpenSearch, Kafka ni
  RabbitMQ.
- No se prioriza escalado horizontal masivo sobre simplicidad y portabilidad.

### Valor diferencial

Demuestra que es posible lograr búsqueda híbrida (vectorial + texto
completo) y una arquitectura de microservicios desacoplados usando
únicamente SQLite (WAL + FTS5 + sqlite-vec) y MQTT como bus de eventos,
manteniendo consumo de recursos mínimo, independencia total entre servicios,
y portabilidad real a hardware modesto.
