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
| SPEC-CONTENT-0001 | done | TASK-CONTENT-0001, TASK-CONTENT-0002, TASK-CONTENT-0003, TASK-CONTENT-0004, TASK-CONTENT-0005 | DEC-0004 | `content-service/*` | `cargo test` (14/14) + walkthrough manual HTTP completo (categoria -> tema -> post -> comentario, compresion ZSTD verificada) | Se corrigio un bug real: violaciones de foreign key se reportaban como `Duplicate`; ahora se distingue por `extended_code` de SQLite |
| SPEC-SEARCH-0001 | done | TASK-SEARCH-0001..0005 (TASK-SEARCH-0006 pendiente, no bloqueante) | DEC-0001, DEC-0002, DEC-0003 | `search-service/*` | `cargo test` (11/11) + walkthrough manual end-to-end real: content-service -> MQTT -> search-service indexa (FTS5 + sqlite-vec) -> GET /search rankea correctamente | EmbeddingProvider con stub hash-based deterministico (decision explicita); dimension 384 igual a MiniLM para reemplazo futuro sin tocar el resto del sistema. Dos bugs reales corregidos: vec0+JOIN requiere `k = ?` explicito; wildcards MQTT no aplican sobre topics separados por `.` |
