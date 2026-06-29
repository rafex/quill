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
state = "todo"
owner = "platform"
dependencies = ["TASK-SEARCH-0002"]
expected_files = ["search-service/src/adapters/onnx_embedding_provider.rs"]
close_criteria = "El mismo texto de prueba produce un embedding semantico real de dimension 384 via ONNX Runtime + MiniLM, sin cambios en ports/application/transport"
validation = ["test que verifica dimension del vector", "medicion documentada de tamano de binario y RAM en frio antes de adoptar en produccion", "comparacion manual: textos semanticamente similares dan mayor similitud que con el stub"]
```

Pendiente de implementar. Riesgo abierto: el binario de ONNX Runtime via
`ort` (feature `download-binaries`) requiere red en el primer build; para
Raspberry Pi/offline-first hay que decidir entre vendoring manual del
binario o aceptar ese costo de build inicial. Esta tarea debe documentar
esa decision antes de cerrarse.

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
