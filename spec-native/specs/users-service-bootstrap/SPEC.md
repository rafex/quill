# SPEC.md

```toml
artifact_type = "spec"
id = "SPEC-USERS-0001"
state = "done"
owner = "platform"
created_at = "2026-06-28"
updated_at = "2026-06-28"
replaces = "none"
related_tasks = ["TASK-USERS-0001", "TASK-USERS-0002", "TASK-USERS-0003", "TASK-USERS-0004"]
related_decisions = []
artifacts = ["users-service/*"]
validation = ["cargo test -p users-service", "manual: init-db + create user + login via HTTP", "manual: publish/consume de un evento MQTT de prueba"]
```

## Metadata

- ID: SPEC-USERS-0001
- Estado: `done`
- Owner: platform
- Fecha de creacion: 2026-06-28
- Ultima actualizacion: 2026-06-28
- Reemplaza: `none`
- Tareas relacionadas: `TASK-USERS-0001`, `TASK-USERS-0002`,
  `TASK-USERS-0003`, `TASK-USERS-0004`
- Decisiones relacionadas: `none`

## Resumen

Crear el esqueleto mínimo de `users-service`: estructura hexagonal completa
(transport/application/domain/ports/adapters/infrastructure), conexión
SQLite propia (`users.sqlite`) con los pragmas obligatorios, un caso de uso
end-to-end (crear usuario) expuesto por Axum, y la base de Inbox/Outbox +
worker MQTT lista para conectarse al bus de eventos. Este servicio es el
primero porque su dominio es el más simple y sirve de plantilla de
estructura para `content-service` y `search-service`.

## Problema

Hoy el repositorio solo tiene documentación (`spec-native/`); no existe
ningún microservicio implementado. Sin un primer servicio de referencia no
hay forma de validar que el patrón de arquitectura (hexagonal + DDD +
inbox/outbox + MQTT) funciona en la práctica antes de replicarlo.

## Objetivo

Tener `users-service` corriendo localmente: expone HTTP (Axum) para crear
y consultar usuarios, persiste en su propia SQLite con WAL, procesa/publica
al menos un mensaje de prueba vía MQTT usando los patrones Inbox y Outbox,
y sigue estrictamente la regla de dependencias hacia el dominio.

## Alcance

- Incluye: estructura de carpetas del servicio, conexión SQLite +
  migraciones iniciales (`init-db`), entidad `User` (dominio), caso de uso
  `CreateUser`, repositorio `UserRepository` (port) + `SqliteUserRepository`
  (adapter), endpoint `POST /users` y `GET /users/:id`, tablas
  `inbox_messages` / `outbox_events`, worker MQTT mínimo (conectar,
  suscribirse, publicar un evento de prueba), CLI básico (`init-db`,
  `migrate`, `stats`).
- Excluye: autenticación real (JWT/sesiones), perfiles avanzados,
  `content-service`, `search-service`, despliegue Docker/k8s, embeddings,
  compresión ZSTD (no aplica a este servicio), observabilidad avanzada
  (OpenTelemetry real, solo dejar el punto de extensión).

## Requisitos funcionales

- RF-1: `POST /users` crea un usuario y persiste en `users.sqlite`.
- RF-2: `GET /users/:id` devuelve el usuario o 404 si no existe.
- RF-3: toda escritura de dominio relevante genera una fila en
  `outbox_events` en la misma transacción (sin publicar aún a MQTT).
- RF-4: un publisher/worker lee `outbox_events` pendientes y publica a
  MQTT después del commit, marcando el evento como publicado.
- RF-5: un mensaje entrante de prueba se registra primero en
  `inbox_messages` antes de procesarse, garantizando idempotencia ante
  duplicados.

## Requisitos no funcionales

- RNF-1: una única conexión SQLite por proceso, con los pragmas
  obligatorios definidos en `STACK.md` (WAL, synchronous=NORMAL,
  foreign_keys=ON, busy_timeout=5000, wal_autocheckpoint=1000).
- RNF-2: el dominio (`domain/`) no importa nada de `adapters/` ni
  `infrastructure/`; verificable por inspección de `use` statements.
- RNF-3: el servicio debe poder arrancar y responder en una máquina con
  recursos equivalentes a una Raspberry Pi (sin medición formal en esta
  iteración, pero sin dependencias pesadas).

## Criterios de aceptacion

- Dado un POST válido a `/users`, cuando se ejecuta, entonces el usuario
  queda persistido y es recuperable vía `GET /users/:id`.
- Dado un caso de uso que modifica el dominio, cuando hace commit,
  entonces existe una fila correspondiente en `outbox_events` y ningún
  mensaje fue publicado a MQTT antes de ese commit.
- Dado un mensaje MQTT de prueba duplicado, cuando llega dos veces,
  entonces se procesa una sola vez (idempotencia vía `inbox_messages`).
- Dado el comando CLI `init-db`, cuando se ejecuta sobre una base nueva,
  entonces se crean todas las tablas necesarias (users, inbox_messages,
  outbox_events) con los pragmas aplicados.

## Dependencias y riesgos

- Dependencia: broker Mosquitto disponible localmente para probar el
  worker MQTT (puede mockearse el cliente para tests unitarios).
- Riesgo: sobre-diseñar el primer servicio. Mitigación: alcance reducido
  a un solo caso de uso real (`CreateUser`) más el andamiaje
  inbox/outbox/worker, sin features de negocio adicionales.
- Riesgo: que la plantilla de carpetas no generalice bien a
  `content-service`/`search-service`. Mitigación: revisar la estructura
  contra `ARCHITECTURE.md` antes de cerrar esta spec.

## Plan de ejecucion

- TASK-USERS-0001: estructura de carpetas + workspace Cargo + conexión
  SQLite con pragmas + `init-db`.
- TASK-USERS-0002: dominio `User` + `UserRepository` (port) +
  `SqliteUserRepository` (adapter) + caso de uso `CreateUser`/`GetUser`.
- TASK-USERS-0003: transport Axum (`POST /users`, `GET /users/:id`) +
  health/ready check.
- TASK-USERS-0004: tablas inbox/outbox + worker MQTT mínimo (rumqttc) +
  publisher de outbox.

## Plan de validacion

- `cargo test -p users-service` (unitarios de dominio + integración con
  SQLite real vía fixtures).
- Walkthrough manual: `init-db`, crear usuario por HTTP, leerlo de vuelta.
- Walkthrough manual: publicar un evento de prueba, verificar que pasa por
  `outbox_events` antes de llegar al broker, y que un duplicado entrante
  no se reprocesa.

## Trazabilidad

- Commits o PRs: pendiente (sin commit todavia en este repo)
- Archivos principales: `users-service/*`
- Resultado de validacion: `cargo test` 11/11 ok; walkthrough manual end-to-end contra Mosquitto real confirmado (HTTP -> outbox -> MQTT -> inbox idempotente)
