# SPEC.md

```toml
artifact_type = "spec"
id = "SPEC-SEARCH-0001"
state = "done"
owner = "platform"
created_at = "2026-06-28"
updated_at = "2026-06-28"
replaces = "none"
related_tasks = ["TASK-SEARCH-0001", "TASK-SEARCH-0002", "TASK-SEARCH-0003", "TASK-SEARCH-0004", "TASK-SEARCH-0005", "TASK-SEARCH-0006"]
related_decisions = ["DEC-0001", "DEC-0002", "DEC-0003", "DEC-0006"]
artifacts = ["search-service/*"]
validation = ["cargo test -p search-service", "manual: indexar un post/comentario real emitido por content-service y verificar fila en sqlite-vec + FTS5", "manual: consulta hibrida con score = 0.60*vector + 0.40*bm25 devuelve resultados coherentes"]
```

## Metadata

- ID: SPEC-SEARCH-0001
- Estado: `done` (TASK-SEARCH-0006 completada el 2026-06-29, ver DEC-0006)
- Owner: platform
- Fecha de creacion: 2026-06-28
- Ultima actualizacion: 2026-06-28
- Reemplaza: `none`
- Tareas relacionadas: `TASK-SEARCH-0001`, `TASK-SEARCH-0002`,
  `TASK-SEARCH-0003`, `TASK-SEARCH-0004`, `TASK-SEARCH-0005`,
  `TASK-SEARCH-0006`
- Decisiones relacionadas: `DEC-0001`, `DEC-0002`, `DEC-0003`, `DEC-0006`

## Resumen

Crear `search-service`, el tercer y más complejo de los microservicios
iniciales del prompt de arquitectura. A diferencia de `users-service`
(SPEC-USERS-0001) y `content-service` (SPEC-CONTENT-0001), este servicio no
expone un dominio de escritura propio: consume eventos de `content-service`
(`forum.post.created`, `forum.comment.created`) vía el patrón Inbox ya
validado, genera embeddings en Rust puro (ONNX Runtime + tokenizers,
modelo `sentence-transformers/all-MiniLM-L6-v2`), los indexa en
`sqlite-vec`, mantiene un índice FTS5 en paralelo, y expone una búsqueda
híbrida (`score = 0.60*vector + 0.40*bm25`).

## Problema

`users-service` y `content-service` ya demuestran que el patrón hexagonal +
SQLite/WAL + Inbox/Outbox + MQTT funciona para servicios que escriben su
propio dominio. Falta demostrar la otra mitad del prompt de arquitectura:
un servicio que **consume** eventos de otros servicios, genera embeddings
sin Python, y combina búsqueda vectorial con búsqueda de texto completo —
todo dentro de SQLite, sin Elasticsearch ni bases vectoriales externas.

## Objetivo

Tener `search-service` corriendo localmente: consume
`forum.post.created`/`forum.comment.created` publicados por
`content-service`, genera un embedding por cada post/comentario con
MiniLM vía ONNX Runtime, lo guarda en `sqlite-vec` (sin duplicar el texto
original más que el snippet necesario para resultados), lo indexa en FTS5,
y responde a consultas híbridas combinando ambos scores.

## Alcance

- Incluye: trait `EmbeddingProvider` + implementación ONNX/MiniLM; schema
  `search.sqlite` con tabla FTS5 y tabla(s) `sqlite-vec`; worker Inbox que
  consume los eventos de contenido y genera el embedding +
  indexación; caso de uso de búsqueda híbrida; endpoint HTTP
  `GET /search?q=...`.
- Excluye: reindexación masiva histórica (se deja como comando CLI
  `reindex` sin implementación completa en esta spec), soporte
  multi-idioma, ranking avanzado más allá de la fórmula fija
  `0.60*vector + 0.40*bm25`, UI de búsqueda.

## Requisitos funcionales

- RF-1: al recibir `forum.post.created` o `forum.comment.created`, el
  servicio genera un embedding del body (vía `EmbeddingProvider`) y lo
  guarda en la tabla `sqlite-vec`, junto con metadata (`model`, `version`,
  `dimension`, `fecha`).
- RF-2: el mismo evento indexa el contenido (title/body o snippet) en una
  tabla virtual FTS5 para búsqueda de texto completo.
- RF-3: `GET /search?q=...` genera el embedding de la consulta, busca en
  `sqlite-vec` y en FTS5, combina resultados con
  `score = 0.60*vector + 0.40*bm25`, y devuelve `id`, `tipo`, `title`,
  `snippet`, `score`.
- RF-4: el patrón Inbox (`inbox_messages`) garantiza que un evento de
  contenido duplicado no se indexa dos veces.

## Requisitos no funcionales

- RNF-1: generación de embeddings 100% en Rust (ONNX Runtime +
  tokenizers); no se invoca Python en ningún punto del flujo de
  producción.
- RNF-2: el cambio de modelo de embeddings no debe requerir tocar nada
  fuera del adapter que implementa `EmbeddingProvider`.
- RNF-3: mismos pragmas SQLite obligatorios que los demás servicios
  (`STACK.md`), una sola conexión por proceso.
- RNF-4: no se duplica el texto original más de lo necesario — el body
  completo vive en `content-service`; `search-service` solo guarda lo
  necesario para mostrar un snippet en resultados.

## Criterios de aceptacion

- Dado un post publicado por `content-service` vía MQTT, cuando
  `search-service` lo consume, entonces existe una fila en la tabla de
  vectores con el embedding y una fila indexada en FTS5.
- Dado el mismo evento de post entregado dos veces (duplicado MQTT),
  cuando se procesa, entonces solo se indexa una vez (idempotencia vía
  `inbox_messages`).
- Dada una consulta de búsqueda, cuando se ejecuta, entonces los
  resultados devueltos incluyen `score` calculado con la fórmula híbrida y
  vienen ordenados de mayor a menor score.

## Dependencias y riesgos

- Dependencia: `content-service` debe estar publicando
  `forum.post.created`/`forum.comment.created` (ya validado en
  SPEC-CONTENT-0001) para que `search-service` tenga algo que indexar en
  las pruebas manuales end-to-end.
- Dependencia externa nueva: ONNX Runtime (crate `ort` o equivalente) y el
  archivo del modelo MiniLM + su tokenizer, que deben poder descargarse o
  empaquetarse para que el servicio sea portable a Raspberry Pi/VPS sin
  acceso a internet en cada arranque.
- Riesgo: el binding de ONNX Runtime en Rust puede traer una dependencia
  nativa pesada (libonnxruntime), lo que tensiona el objetivo de "bajo
  consumo de recursos / portable a Raspberry Pi". Mitigación: medir el
  tamaño del binario y el uso de RAM en la primera iteración antes de
  comprometerse a esa ruta; si el costo es alto, documentar la
  alternativa (modelo más pequeño o cuantizado) como decisión explícita.
- Riesgo: `sqlite-vec` es una extensión de SQLite relativamente nueva;
  validar que el crate de Rust disponible carga la extensión de forma
  confiable en modo bundled antes de comprometer el resto del diseño.
- Confirmado: `ort` (ONNX Runtime), `sqlite-vec` (v0.1.9) y `tokenizers`
  existen en crates.io y son instalables en Rust estable. `ort` resuelve
  el binario nativo de ONNX Runtime vía la feature `download-binaries`
  (binario prebuilt por plataforma), lo que implica que el primer build
  necesita acceso a red o vendoring manual del binario para entornos
  offline-first/Raspberry Pi sin internet — esto se documenta como
  decision pendiente para TASK-SEARCH-0002, no se resuelve en esta spec.
- Riesgo: mezclar mal los pesos de la fórmula híbrida puede dar resultados
  pobres. Mitigación: mantener `0.60/0.40` como constantes nombradas y
  fáciles de ajustar, no hardcodeadas en múltiples lugares.

## Plan de ejecucion

- TASK-SEARCH-0001: esqueleto del servicio + `search.sqlite` (pragmas,
  tabla FTS5, tabla(s) de vectores, `inbox_messages`, `outbox_events`
  aunque este servicio no necesite outbox propio inicialmente).
- TASK-SEARCH-0002: trait `EmbeddingProvider` + implementación ONNX/MiniLM,
  probada con un texto fijo y un embedding de dimensión conocida.
- TASK-SEARCH-0003: integración `sqlite-vec` — insertar y consultar
  vectores por similitud, con metadata de modelo/version/dimension/fecha.
- TASK-SEARCH-0004: worker Inbox que consume
  `forum.post.created`/`forum.comment.created`, genera embedding e indexa
  en FTS5 + vectores, con idempotencia probada.
- TASK-SEARCH-0005: caso de uso de búsqueda híbrida + endpoint
  `GET /search?q=...`.
- TASK-SEARCH-0006: reemplazar el stub de TASK-SEARCH-0002 por la
  integración real ONNX Runtime + MiniLM, decidida explícitamente como
  tarea separada (no bloqueante para validar el resto del pipeline).

## Plan de validacion

- `cargo test -p search-service`.
- Walkthrough manual: levantar Mosquitto + `content-service`, crear un
  post real, correr `publish-outbox` de `content-service`, levantar el
  worker de `search-service` y confirmar que el post queda indexado
  (fila en vectores + fila en FTS5).
- Walkthrough manual: `GET /search?q=...` con una consulta relacionada al
  post de prueba y confirmar que aparece en los resultados con un score
  coherente.

## Trazabilidad

- Commits o PRs: pendiente (sin commit todavia en este repo)
- Archivos principales: `search-service/*`
- Resultado de validacion: `cargo test` 11/11 ok (tambien con
  `--features onnx-embeddings`); walkthrough manual end-to-end real:
  content-service crea posts -> publish-outbox -> search-service indexa
  via MQTT (FTS5 + sqlite-vec real) -> GET /search devuelve resultados
  rankeados correctamente por relevancia textual y, con el provider ONNX
  real, tambien por relevancia semantica (consulta sin overlap lexico
  encuentra el contenido correcto). Bugs reales encontrados y
  corregidos: (1) vec0 con JOIN requiere `k = ?` explicito en WHERE en
  vez de LIMIT; (2) los wildcards MQTT (`+`/`#`) solo aplican a niveles
  separados por `/`, no por `.`, asi que hubo que suscribirse a cada
  topic de contenido explicitamente; (3) API real de `ort` 2.0.0-rc.12
  difiere de la documentacion generica usada inicialmente (tensores via
  tuplas `(shape, Vec<T>)`, no `ndarray` directo).
