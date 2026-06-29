# Search Service Bootstrap

Tercera iniciativa de implementación. A diferencia de `users-service` y
`content-service`, este servicio no escribe su propio dominio: consume
eventos de contenido, genera embeddings en Rust (ONNX + MiniLM), indexa en
FTS5 + sqlite-vec, y expone búsqueda híbrida.

## Documentos

- `SPEC.md`
- `../../tasks/search-service-bootstrap/TASKS.md`
- `../../TRACEABILITY.md`
