# DECISIONS.md

Registro de decisiones persistentes del proyecto.

## Cuando registrar aqui

Registrar una decision cuando cambie algo que futuras iniciativas
o agentes deban respetar:

- la arquitectura del sistema
- una convencion de codigo o de documentacion
- una tecnologia o dependencia base
- un tradeoff que condicione trabajo futuro

Ver `AGENTS.md` para entender la separacion semantica entre este
archivo y `SPEC.md`.

## Cuando leer este archivo

Antes de iniciar una nueva iniciativa, revisar si alguna decision
registrada condiciona el diseno o la implementacion.

## Decisiones

### DEC-0001 - Stub de embeddings determinista antes de ONNX real

- Fecha: 2026-06-28
- Estado: `accepted`
- Relacionado con specs: SPEC-SEARCH-0001
- Relacionado con tareas: TASK-SEARCH-0002, TASK-SEARCH-0006
- Contexto: integrar ONNX Runtime + MiniLM de inmediato introduce riesgo
  de portabilidad (el crate `ort` resuelve el binario nativo vía
  `download-binaries`, lo que requiere red o vendoring en el primer
  build) sin haber validado aún el resto del pipeline de búsqueda
  híbrida.
- Decision: definir el trait `EmbeddingProvider` y una implementación
  stub hash-based determinista (dimensión 384, igual a MiniLM) para
  construir y probar el resto del sistema (vector store, worker de
  indexación, búsqueda híbrida) sin bloquear en la integración ONNX.
  La integración real queda en `TASK-SEARCH-0006`, un reemplazo de
  adapter sin tocar `ports`/`application`/`transport`.
- Consecuencias: el ranking semántico actual no es real (es hash-based),
  solo demuestra el mecanismo. El score combinado en producción no debe
  usarse hasta que TASK-SEARCH-0006 se complete.
- Reemplaza: `none`

### DEC-0002 - sqlite-vec real en vez de similitud manual

- Fecha: 2026-06-28
- Estado: `accepted`
- Relacionado con specs: SPEC-SEARCH-0001
- Relacionado con tareas: TASK-SEARCH-0003
- Contexto: `sqlite-vec` es una extensión de carga dinámica; antes de
  comprometerse a usarla había que validar que el crate de Rust la
  carga de forma confiable en modo bundled.
- Decision: usar `sqlite-vec` real (tabla virtual `vec0`) en vez de
  calcular similitud manualmente en Rust. Se validó con un probe
  aislado fuera del proyecto antes de integrarla. Hallazgo durante la
  integración: con `JOIN`, `vec0` exige la restricción `k = ?` explícita
  en el `WHERE` en vez de un `LIMIT` al final de la consulta.
- Consecuencias: el servicio depende de una extensión SQLite de
  terceros relativamente nueva; si en el futuro deja de mantenerse,
  habría que volver a evaluar similitud manual como fallback.
- Reemplaza: `none`

### DEC-0003 - Topics MQTT no soportan wildcards de nivel con `.`

- Fecha: 2026-06-28
- Estado: `accepted`
- Relacionado con specs: SPEC-SEARCH-0001
- Relacionado con tareas: TASK-SEARCH-0004
- Contexto: se intentó suscribir a `forum.+.created` asumiendo que `+`
  actúa como wildcard de nivel igual que en convenciones con `/`. MQTT
  define los wildcards `+`/`#` únicamente sobre niveles separados por
  `/`; con `.` como separador, todo el string es un único nivel.
- Decision: cada microservicio que necesite consumir múltiples topics
  de contenido debe suscribirse explícitamente a cada topic completo
  (sin asumir wildcards), por ejemplo
  `forum.post.created` y `forum.comment.created` por separado.
- Consecuencias: agregar un nuevo tipo de contenido a indexar requiere
  agregar su topic explícito a la lista de suscripción, no se cubre
  automáticamente.
- Reemplaza: `none`

### DEC-0004 - Distinguir violaciones de FK de violaciones de unicidad en SQLite

- Fecha: 2026-06-28
- Estado: `accepted`
- Relacionado con specs: SPEC-CONTENT-0001
- Relacionado con tareas: TASK-CONTENT-0003
- Contexto: el primer `map_insert_error` de los adapters SQLite
  clasificaba cualquier `rusqlite::ErrorCode::ConstraintViolation` como
  `RepositoryError::Duplicate`, lo cual confundía una violación de
  foreign key (p. ej. crear un post con un `topic_id` inexistente) con
  un duplicado real de slug/email.
- Decision: distinguir por `extended_code` de SQLite
  (`SQLITE_CONSTRAINT_UNIQUE` / `SQLITE_CONSTRAINT_PRIMARYKEY` mapean a
  `Duplicate`; cualquier otro `ConstraintViolation`, incluyendo FK, cae
  en `Unknown`). Aplicado de forma consistente en los cuatro adapters de
  `content-service`.
- Consecuencias: ningún caso de uso debe asumir que `Duplicate` cubre
  errores de integridad referencial; ese caso ahora se reporta como
  error genérico con el mensaje real de SQLite.
- Reemplaza: `none`
