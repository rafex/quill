# Template Project Agents AI

Framework de spec-driven development para operar repos AI-native.
La navegacion empieza por carpeta, no por un unico documento global.

Esta version `v0.3` agrega:

- contrato documental explicito en `.specnative/SCHEMA.md`
- estados obligatorios para specs, tareas y decisiones
- trazabilidad entre artefactos
- capa de ejecucion con `tasks/`
- workflows operativos repetibles
- ejemplo end-to-end por iniciativa
- soporte opcional para metadata parseable en TOML
- CLI externo para validacion y exportacion (ver `.specnative/CLI.md`)

## Principios

- Los `README.md` de cada carpeta son el punto de entrada y el indice
  de navegacion.
- Los archivos en MAYUSCULAS son contexto operativo para agentes.
- Cada verdad vive en un solo documento. No duplicar entre archivos.
- Leer el minimo contexto suficiente para ejecutar bien la tarea.
- Toda iniciativa relevante debe poder trazarse desde spec hasta
  validacion.

## Documentos del proyecto

- `PRODUCT.md`: problema, usuarios, objetivos y alcance.
- `SPEC.md`: capacidad o cambio que debe implementarse.
- `DECISIONS.md`: decisiones relevantes y sus tradeoffs.
- `ARCHITECTURE.md`, `STACK.md`, `CONVENTIONS.md`, `COMMANDS.md`:
  restricciones operativas del sistema.
- `ROADMAP.md`: direccion temporal, sin detalle de implementacion.
- `TRACEABILITY.md`: relaciones entre specs, tareas, decisiones y
  validacion.

Todos estos documentos viven dentro de `agents/`.

## Estructura

- [`AGENTS.md`](./AGENTS.md):
  contrato operativo para agentes — flujo de trabajo, politicas y
  criterios de actualizacion.
- [`agents/README.md`](./agents/README.md):
  indice principal del sistema de contexto.
- [`tasks/README.md`](./tasks/README.md):
  indice del sistema de ejecucion y estado de tareas.
- [`workflows/README.md`](./workflows/README.md):
  procedimientos repetibles para planificar, implementar y validar.
- [`pipelines/README.md`](./pipelines/README.md):
  contexto de integracion continua y entrega continua.
- [`.specnative/README.md`](./.specnative/README.md):
  documentacion del framework y referencias al CLI.

## Como usar este template

1. Copiar este template a un repo nuevo.
2. Leer `AGENTS.md` para entender el contrato operativo.
3. Completar los documentos base dentro de `agents/`.
4. Crear specs en `agents/specs/` o en `agents/SPEC.md`.
5. Derivar tareas ejecutables en `tasks/`.
6. Consultar [`.specnative/README.md`](./.specnative/README.md) para
   usar el CLI del framework opcionalmente.

## Regla de separacion

- `agents/`: contexto del proyecto adoptante.
- `tasks/`: plan ejecutable derivado de specs.
- `workflows/`: procedimientos operativos repetibles.
- `pipelines/`: contexto de CI/CD del proyecto.
- `.specnative/`: documentacion del framework (no del proyecto).
