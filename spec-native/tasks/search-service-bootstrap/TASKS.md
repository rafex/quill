# TASKS.md

```toml
artifact_type = "task_file"
initiative = "search-service-bootstrap"
spec_id = "SPEC-SEARCH-0001"
owner = "platform"
state = "done"
```

## Metadata

- Iniciativa: search-service-bootstrap
- Spec relacionada: SPEC-SEARCH-0001
- Owner: platform
- Estado general: `done` (TASK-SEARCH-0006 sigue `todo` como follow-up no bloqueante)

## Tareas

### TASK-SEARCH-0001 - Esqueleto del servicio y search.sqlite

```toml
id = "TASK-SEARCH-0001"
title = "Esqueleto del servicio y search.sqlite"
state = "done"
owner = "platform"
dependencies = []
expected_files = ["search-service/Cargo.toml", "search-service/src/infrastructure/db.rs", "search-service/src/main.rs"]
close_criteria = "init-db crea search.sqlite con WAL, tabla virtual FTS5, tabla(s) de vectores e inbox_messages"
validation = ["test de integracion: pragmas aplicados y tablas creadas", "walkthrough manual de init-db"]
```

Misma estructura de capas que `users-service`/`content-service`. El
schema incluye una tabla FTS5 (`content_fts`) y una tabla de vectores
(definida en TASK-SEARCH-0003), más `inbox_messages` para idempotencia.

### TASK-SEARCH-0002 - EmbeddingProvider (stub deterministico)

```toml
id = "TASK-SEARCH-0002"
title = "EmbeddingProvider con stub deterministico (hash-based)"
state = "done"
owner = "platform"
dependencies = ["TASK-SEARCH-0001"]
expected_files = ["search-service/src/ports/embedding_provider.rs", "search-service/src/adapters/stub_embedding_provider.rs"]
close_criteria = "Un texto fijo de prueba produce un embedding de dimension conocida (384, igual a MiniLM) y estable entre llamadas"
validation = ["test que verifica dimension del vector", "test que verifica determinismo (mismo texto -> mismo vector)", "test que verifica que textos distintos producen vectores distintos"]
```

Decision tomada explicitamente con el desarrollador: en vez de integrar
ONNX Runtime + MiniLM de inmediato (riesgo de portabilidad documentado en
la spec, requiere red/vendoring para el binario nativo), se define el
trait `EmbeddingProvider` y una implementacion stub hash-based
determinista con dimension 384 (igual a MiniLM) para poder construir y
probar TASK-SEARCH-0003/0004/0005 end-to-end ya. La integracion real con
ONNX/MiniLM queda como TASK-SEARCH-0006, un reemplazo de adapter sin tocar
el resto del sistema (ese es el punto de tener el trait).

### TASK-SEARCH-0006 - EmbeddingProvider real (ONNX + MiniLM)

```toml
id = "TASK-SEARCH-0006"
title = "Reemplazar el stub por ONNX Runtime + MiniLM"
state = "done"
owner = "platform"
dependencies = ["TASK-SEARCH-0002"]
expected_files = ["search-service/src/adapters/onnx_embedding_provider.rs", "search-service/src/adapters/model_downloader.rs", "search-service/Cargo.toml", "search-service/src/main.rs"]
close_criteria = "El mismo texto de prueba produce un embedding semantico real de dimension 384 via ONNX Runtime + MiniLM, sin cambios en ports/application/transport. La eleccion stub/onnx es de build (Cargo feature) y runtime (env var), no una limitante fija."
validation = ["cargo test -p search-service y --features onnx-embeddings (11/11 ambos)", "medicion documentada de tamano de binario (4.9MB vs 33MB) y RAM en frio (~90MB) - ver DEC-0006", "validacion semantica real: consulta sin overlap lexico encuentra el post correcto (probado con Mosquitto + content-service reales)"]
```

Completado. Se agregaron Cargo features `stub-embeddings` (default) y
`onnx-embeddings` (opcional: `ort`, `tokenizers`, `ureq`) en
`search-service/Cargo.toml`. La seleccion en runtime es
`SEARCH_EMBEDDING_PROVIDER=stub|onnx`. El modelo
(`Xenova/all-MiniLM-L6-v2` quantizado) se descarga con el comando
explicito `download-model`, no automaticamente. Ver DEC-0006 en
`DECISIONS.md` para las mediciones de tamano de binario/RAM y la
validacion semantica real contra Mosquitto + content-service.

### TASK-SEARCH-0003 - Integracion sqlite-vec

```toml
id = "TASK-SEARCH-0003"
title = "Insertar y consultar vectores con sqlite-vec"
state = "done"
owner = "platform"
dependencies = ["TASK-SEARCH-0002"]
expected_files = ["search-service/src/infrastructure/vector_store.rs"]
close_criteria = "Un vector insertado es recuperable por similitud (top-k) junto con su metadata de modelo/version/dimension/fecha"
validation = ["test de integracion: insertar N vectores y verificar orden de similitud esperado"]
```

### TASK-SEARCH-0004 - Worker Inbox de indexacion

```toml
id = "TASK-SEARCH-0004"
title = "Worker Inbox que indexa posts y comentarios"
state = "done"
owner = "platform"
dependencies = ["TASK-SEARCH-0003"]
expected_files = ["search-service/src/application/index_content.rs", "search-service/src/main.rs"]
close_criteria = "Un mensaje forum.post.created o forum.comment.created genera embedding + fila FTS5 + fila de vector; un duplicado no se reprocesa"
validation = ["test de idempotencia con mensaje duplicado", "walkthrough manual contra Mosquitto + content-service real"]
```

### TASK-SEARCH-0005 - Busqueda hibrida (HTTP)

```toml
id = "TASK-SEARCH-0005"
title = "Caso de uso y endpoint de busqueda hibrida"
state = "done"
owner = "platform"
dependencies = ["TASK-SEARCH-0004"]
expected_files = ["search-service/src/application/hybrid_search.rs", "search-service/src/transport/http.rs"]
close_criteria = "GET /search?q=... combina vector y FTS5 con score = 0.60*vector + 0.40*bm25 y devuelve resultados ordenados por score"
validation = ["test unitario de la formula de combinacion de scores", "walkthrough manual end-to-end: indexar contenido real y buscarlo"]
```

### TASK-SEARCH-0007 - Upsert por ext_id (soporte de reindex)

```toml
id = "TASK-SEARCH-0007"
title = "IndexContent upsert por ext_id + message_id por event_id"
state = "done"
owner = "platform"
dependencies = ["TASK-SEARCH-0004"]
expected_files = ["search-service/src/infrastructure/vector_store.rs", "search-service/src/application/index_content.rs", "search-service/src/main.rs"]
close_criteria = "Reindexar el mismo contenido (forum.search.reindex.request en content-service) no duplica filas en embeddings/content_fts, y el mensaje se reprocesa (no se descarta como duplicado)"
validation = ["test: reindexar el mismo ext_id no duplica filas", "walkthrough manual end-to-end con content-service: indexar, reindexar, confirmar conteo de filas sin cambios"]
```

### TASK-SEARCH-0008 - forum.embedding.generate.request / forum.embedding.generated

```toml
id = "TASK-SEARCH-0008"
title = "Topic de generacion de embeddings bajo demanda"
state = "done"
owner = "platform"
dependencies = ["TASK-SEARCH-0002"]
expected_files = ["search-service/src/main.rs"]
close_criteria = "Un mensaje en forum.embedding.generate.request con {request_id, text} genera un embedding y publica la respuesta en forum.embedding.generated con {request_id, model, version, dimension, embedding}"
validation = ["build limpio (14/14 tests)", "walkthrough manual: publicar un request con mosquitto_pub, confirmar respuesta en forum.embedding.generated con mosquitto_sub"]
```

Desacopla la generacion de embeddings del pipeline de indexacion: cualquier
servicio puede solicitar un embedding via MQTT sin conocer el provider concreto
(stub vs ONNX). No usa inbox dedup (ver DEC-0009): la generacion es stateless e
idempotente, reintentar produce el mismo resultado. El proceso de inbox existente
se refactorizo en dos funciones separadas (`handle_index_request` /
`handle_embedding_request`) para mantener cada handler legible.

Bug latente corregido en el camino: `IndexContent::execute` hacia
`INSERT` ciego, asi que cualquier redelivery de `forum.post.created`
para el mismo post ya habria duplicado filas, incluso sin reindex
deliberado. Ver `DEC-0008`.
