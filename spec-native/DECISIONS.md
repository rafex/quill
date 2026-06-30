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
- Actualizacion 2026-06-29: TASK-SEARCH-0006 se completó. Ver DEC-0006
  para la decisión de cómo se integró ONNX real (como eleccion de build,
  no como reemplazo obligatorio del stub).

### DEC-0006 - ONNX/MiniLM real como Cargo feature, eleccion de build/runtime

- Fecha: 2026-06-29
- Estado: `accepted`
- Relacionado con specs: SPEC-SEARCH-0001
- Relacionado con tareas: TASK-SEARCH-0006
- Contexto: tras DEC-0001, había que decidir cómo convive el stub
  determinista con la integración ONNX real sin que una excluya a la
  otra — el usuario pidió explícitamente que fuera "una elección de
  instalación, no una limitante".
- Decision: `search-service` expone dos Cargo features:
  `stub-embeddings` (default) y `onnx-embeddings` (trae `ort`,
  `tokenizers`, `ureq` como dependencias opcionales). El binario
  compilado sin `onnx-embeddings` no tiene ninguna dependencia nativa de
  ONNX. En runtime, la variable `SEARCH_EMBEDDING_PROVIDER=stub|onnx`
  elige el adapter; si se pide `onnx` sin haber compilado con esa
  feature, falla con un mensaje explícito (no falla en silencio). El
  modelo (`Xenova/all-MiniLM-L6-v2` quantizado, ONNX) y su tokenizer se
  descargan con el comando explícito `download-model` (vía `ureq`), no
  automáticamente en cada arranque — así un despliegue offline/Raspberry
  Pi puede copiar los archivos del modelo manualmente sin necesitar red
  en el servicio.
- Medido (macOS arm64, build release):
  - Binario: 4.9 MB (`stub-embeddings`, default) vs 33 MB
    (`onnx-embeddings`) — el runtime de ONNX queda enlazado
    estáticamente (`libonnxruntime.a`, ~80 MB sin comprimir, cacheado
    fuera del repo).
  - Modelo descargado: ~22.5 MB (tokenizer.json + model_quantized.onnx).
  - RSS en frío con el provider ONNX cargado (antes de cualquier
    consulta): ~90 MB. Tras una consulta real con inferencia: ~92 MB.
  - Validado semánticamente: una consulta sin ninguna palabra en común
    con el post indexado (`"vehicles and automobiles"` vs un post que
    dice `"cars"`/`"driving"`/`"highway"`) lo encuentra y lo rankea
    primero — algo que el stub hash-based nunca podía lograr.
- Consecuencias: ~90MB de RSS es significativamente más que los otros
  servicios (sin medir aún en Raspberry Pi real, solo macOS arm64); si
  ese costo resulta inaceptable en hardware objetivo, la mitigación es
  seguir usando `stub-embeddings` (build sin la feature) o evaluar un
  modelo aún más pequeño, no volver a Python.
- Reemplaza: `none` (complementa DEC-0001; el stub sigue siendo el
  default y la opción recomendada para builds ultra-ligeros).

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

### DEC-0005 - Se descarta NATS como bus de eventos

- Fecha: 2026-06-28
- Estado: `accepted`
- Relacionado con specs: SPEC-USERS-0001, SPEC-CONTENT-0001, SPEC-SEARCH-0001
- Relacionado con tareas: `none`
- Contexto: se evaluó reemplazar MQTT/Mosquitto por NATS (con JetStream)
  como bus de eventos entre microservicios. NATS/JetStream ofrece
  persistencia y entrega at-least-once a nivel de broker, lo que podría
  simplificar o reemplazar parte del patrón Inbox/Outbox implementado a
  mano en SQLite.
- Decision: mantener MQTT/Mosquitto. Motivos: (1) MQTT ya está validado
  end-to-end con un broker Mosquitto real en los tres microservicios
  (`users-service`, `content-service`, `search-service`); (2) Mosquitto
  tiene una huella de recursos significativamente menor que un servidor
  NATS, y esa huella mínima en Raspberry Pi/VPS pequeños es una
  restricción explícita documentada en `STACK.md` y `PRODUCT.md`, no un
  detalle de implementación negociable; (3) el costo de migrar (rehacer
  `infrastructure/mqtt.rs` en los tres servicios y volver a validar todo
  el pipeline) no se justifica sin una razón concreta que lo amerite.
- Consecuencias: el patrón Inbox/Outbox en SQLite sigue siendo
  responsabilidad de cada servicio (no se delega al broker). Si en el
  futuro ese patrón se vuelve costoso de mantener o aparece un requisito
  real de persistencia/replay a nivel de broker, esta decision debe
  revisarse explícitamente — no descartar NATS por inercia sin releer
  este registro.
- Reemplaza: `none`

### DEC-0007 - Reintentos con backoff fijo antes de forum.deadletter

- Fecha: 2026-06-29
- Estado: `accepted`
- Relacionado con specs: SPEC-USERS-0001, SPEC-CONTENT-0001, SPEC-SEARCH-0001
- Relacionado con tareas: `none` (trabajo transversal, no ligado a una sola spec)
- Contexto: el worker Inbox de los tres servicios fallaba en silencio
  (solo `eprintln!`) cuando el handler devolvía error; el mensaje se
  perdía sin rastro y sin ningún mecanismo de recuperación, pese a que
  `forum.deadletter` ya estaba documentado como topic esperado desde el
  prompt de arquitectura original.
- Decision: se agregó `infrastructure::inbox_worker::process_with_retry`
  (idéntico en los tres servicios, igual que el resto de `inbox_worker.rs`)
  con un backoff fijo de `[200ms, 500ms, 1000ms]` (3 reintentos) antes de
  rendirse. Es seguro reintentar porque `process_message` ya distingue
  "recibido pero no procesado" de "procesado": un reintento nunca
  reinserta ni reprocesa una fila ya confirmada. Al agotar los reintentos,
  se publica a `forum.deadletter` un envelope con `original_topic`,
  `message_id`, `error` y el `payload` original completo, usando el
  mismo cliente MQTT que ya estaba abierto para suscribirse (no se abre
  una conexión nueva).
- Consecuencias: el backoff es fijo y deliberadamente corto (no modela
  un SLA real); si en el futuro se necesita backoff exponencial
  configurable o un límite de reintentos distinto por servicio, hay que
  revisar esta decisión. Nadie consume `forum.deadletter` todavía — hoy
  es solo un registro para inspección manual, no hay un proceso de
  reprocesamiento automático de la cola de deadletter.
- Validado contra Mosquitto real: un `forum.post.create.request` con un
  `topic_id` inexistente (viola la foreign key) reintenta 3 veces con
  los delays esperados y termina publicando a `forum.deadletter` con el
  error real de SQLite.
- Reemplaza: `none`

### DEC-0009 - forum.embedding.generate.request sin inbox dedup

- Fecha: 2026-06-29
- Estado: `accepted`
- Relacionado con specs: SPEC-SEARCH-0001
- Relacionado con tareas: TASK-SEARCH-0008
- Contexto: el topic `forum.embedding.generate.request` / `forum.embedding.generated`
  desacopla la generación de embeddings del pipeline de indexación. A diferencia de los
  demás handlers de `process-inbox`, la generación de embeddings es **stateless e
  idempotente por naturaleza**: el mismo texto siempre produce el mismo embedding
  (para el mismo provider/modelo). Guardar el `message_id` en `inbox_messages` para
  este topic solo añadiría escrituras sin aportar garantía real de corrección.
- Decision: `handle_embedding_request` no usa el patrón Inbox (no llama a
  `process_with_retry` ni a `inbox_messages`). Si el broker reentrega el mismo
  request, se vuelve a generar y publicar el mismo embedding — efecto idéntico al
  resultado deseado. El costo de un reintento es una inferencia de embedding extra
  (barato con el stub; aceptable con ONNX en ausencia de carga alta).
- Consecuencias: si en el futuro la generación de embeddings tiene efectos
  secundarios (p. ej. facturación por llamada a una API externa), habrá que agregar
  dedup. Hoy no aplica.
- Reemplaza: `none`

### DEC-0008 - event_id por emision + upsert por ext_id para soportar reindex

- Fecha: 2026-06-29
- Estado: `accepted`
- Relacionado con specs: SPEC-CONTENT-0001, SPEC-SEARCH-0001
- Relacionado con tareas: `none` (trabajo transversal)
- Contexto: para implementar `forum.search.reindex.request` se necesitaba
  que `content-service` pudiera re-publicar `forum.post.created`/
  `forum.comment.created` para contenido ya existente. Esto choca con dos
  cosas ya construidas: (1) el `message_id` del Inbox de `search-service`
  se derivaba del `id` del contenido, así que una republicación del mismo
  post se ignoraba como duplicado; (2) `IndexContent::execute` hacía un
  `INSERT` ciego en `embeddings`/`content_fts`, así que si lograba pasar
  el Inbox, habría generado filas duplicadas para el mismo `ext_id`.
- Decision: cada fila de `outbox_events` ahora lleva su propio `event_id`
  (distinto del `id` del post/comentario) incluido en el payload
  publicado; `search-service` deriva el `message_id` del Inbox de ese
  `event_id` (con fallback al `id` del contenido si falta, por si llega
  un payload de otro origen sin ese campo). `IndexContent::execute` pasa
  a ser upsert por `ext_id`: borra la fila vieja de `embeddings`/
  `vec_items`/`content_fts` antes de insertar la nueva
  (`VectorStore::delete_by_ext_id`).
- Consecuencias: este upsert corrige también un bug latente que ya
  existía antes del reindex — cualquier redelivery de
  `forum.post.created` para el mismo post (aunque no fuera un reindex
  deliberado) ya habría producido filas duplicadas en `search.sqlite`.
- Validado contra Mosquitto real: indexar un post, pedir
  `forum.search.reindex.request`, publicar y consumir de nuevo —
  `embeddings`/`content_fts` se mantienen en 1 fila por post (no 2), y
  el log muestra `indexed` (reprocesado), no `skipped`.
- Reemplaza: `none`
