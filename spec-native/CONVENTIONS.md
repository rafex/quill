# CONVENTIONS.md

Reglas operativas y de implementacion del proyecto.

### Codigo

- Cada microservicio sigue siempre la estructura:
  `transport / application / domain / ports / adapters / infrastructure`.
- La dependencia de código siempre apunta hacia el dominio; nunca
  dependencias inversas (domain no conoce adapters/infrastructure).
- Todo el SQL se escribe manualmente; prohibido usar ORM.
- Los repositorios son siempre interfaces (ports); la implementación
  concreta es `SQLiteRepository` en `adapters`.
- Los Use Cases nunca acceden a SQLite directamente; los handlers HTTP
  nunca acceden al repositorio directamente, solo a casos de uso.
- Cambiar el modelo de embeddings no debe requerir tocar nada fuera del
  adapter que implementa `EmbeddingProvider`.
- Preferir cambios pequeños y locales; evitar duplicación accidental.

### Tests

- Tests unitarios para domain/application de cada servicio.
- Tests de integración con fixtures SQLite reales (no mocks de SQLite).
- Benchmarks para rutas críticas (compresión, embeddings, búsqueda
  híbrida).
- Cada cambio relevante debe definir su estrategia de validación.

### Desarrollo iterativo

- No generar todo el código de una sola vez. Trabajar iterativamente.
- Para cada iteración: (1) explicar la decisión de diseño, (2) generar
  únicamente los archivos necesarios, (3) esperar validación antes de
  continuar.
- Prioridad siempre: simplicidad, mantenibilidad, portabilidad, bajo
  consumo de recursos, código idiomático en Rust, separación estricta de
  responsabilidades, independencia completa entre microservicios.

### Documentacion

- Los `README.md` indexan.
- Los archivos en MAYUSCULAS contienen contexto fuente.
- No duplicar hechos entre documentos sin una razon fuerte.
- Generar y mantener: README, diagrama C4, diagrama de componentes,
  diagrama de secuencia (creación de post), diagrama de búsqueda híbrida,
  y explicación de decisiones arquitectónicas.

### Agentes

- Antes de editar, leer el `README.md` de la carpeta.
- Actualizar el documento fuente si cambia una verdad compartida.
- No cerrar una tarea sin estado final y evidencia de validacion.
- No ejecutar una iniciativa sin referencia explicita a una spec.
- Nunca permitir que un microservicio acceda a la base SQLite de otro
  servicio, ni en código de producción ni en tests de integración.
