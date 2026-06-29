# TRACEABILITY.md

Mapa de relaciones entre specs, tareas, decisiones y validacion.

## Objetivo

Permitir que una persona o agente pueda reconstruir rapidamente:

- que spec origino un cambio
- que tareas ejecutaron esa spec
- que decisiones condicionaron el trabajo
- que evidencia valida el resultado

## Cuando actualizar este archivo

Actualizar al cerrar una iniciativa, no durante la ejecucion.
El momento correcto es cuando la spec pasa a estado `done` o `blocked`.

Si una decision cambia el alcance de una spec activa, registrar
la relacion antes de continuar.

## Formato sugerido

| Spec | Estado | Tareas | Decisiones | Archivos principales | Validacion | Observaciones |
| --- | --- | --- | --- | --- | --- | --- |
| SPEC-0001 | done | TASK-0001, TASK-0002 | DEC-0001 | `src/auth/*` | `npm test` | |
| SPEC-USERS-0001 | done | TASK-USERS-0001, TASK-USERS-0002, TASK-USERS-0003, TASK-USERS-0004 | none | `users-service/*` | `cargo test` (11/11) + walkthrough manual contra Mosquitto real (HTTP -> outbox -> MQTT -> inbox idempotente) | Plantilla de referencia hexagonal para content-service y search-service |
| SPEC-CONTENT-0001 | done | TASK-CONTENT-0001..0006 (todas completas) | DEC-0004 | `content-service/*` | `cargo test` (14/14) + walkthrough manual HTTP completo (categoria -> tema -> post -> comentario, compresion ZSTD verificada) + walkthrough manual via MQTT puro (forum.post.create.request/forum.comment.create.request crean recursos reales, idempotencia por request_id confirmada) | Se corrigio un bug real: violaciones de foreign key se reportaban como `Duplicate`; ahora se distingue por `extended_code` de SQLite. TASK-CONTENT-0006 reemplaza el placeholder forum.content.command por topics reales conectados a CreatePost/CreateComment |
| SPEC-SEARCH-0001 | done | TASK-SEARCH-0001..0006 (todas completas) | DEC-0001, DEC-0002, DEC-0003, DEC-0006 | `search-service/*` | `cargo test` (11/11, default y `--features onnx-embeddings`) + walkthrough manual end-to-end real con Mosquitto/content-service, incluyendo busqueda semantica real (consulta sin overlap lexico encuentra el contenido correcto) | EmbeddingProvider con dos adapters: stub hash-based (default, sin deps nativas) y ONNX/MiniLM real (Cargo feature `onnx-embeddings`, eleccion via `SEARCH_EMBEDDING_PROVIDER`). Binario 4.9MB vs 33MB; RSS ~90MB con ONNX cargado. Tres bugs reales corregidos: vec0+JOIN requiere `k = ?` explicito; wildcards MQTT no aplican sobre topics separados por `.`; API real de `ort` 2.0.0-rc.12 usa tuplas `(shape, Vec<T>)`, no `ndarray` directo |
