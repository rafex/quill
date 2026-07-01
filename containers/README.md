# Probar Quill en local con contenedores

Esta guía levanta el foro ligero completo — broker MQTT, los tres
servicios Rust y el frontend web — usando Podman (o Docker) Compose.
Es la forma más rápida de ver el sistema end-to-end sin instalar Rust,
Node ni dependencias nativas en la máquina.

## Requisitos

- [Podman](https://podman.io/) + [podman-compose](https://github.com/containers/podman-compose),
  o Docker + Docker Compose como alternativa.
- `curl` y `jq` (opcional, solo para los smoke tests).
- Si usas las recipes de `just`, instala [just](https://github.com/casey/just).

El `Justfile` del repo detecta automáticamente cuál tienes instalado:
usa `podman-compose` si existe, si no cae a `docker compose`.

## Qué se levanta

| Servicio         | Puerto host | Rol                                                    |
|------------------|-------------|---------------------------------------------------------|
| `mosquitto`      | 1883        | Broker MQTT (bus de eventos entre servicios)            |
| `users-service`  | 8080        | Alta de usuarios                                         |
| `content-service`| 8081        | Categorías, temas, posts, comentarios + SSE en `/events` |
| `search-service` | 8082        | Búsqueda híbrida (BM25 + vectores, embeddings stub)       |
| `web`            | 8090        | Frontend estático (TypeScript + WebAssembly) servido por Nginx |

Cada servicio Rust corre con un único contenedor en modo dev: hace
`init-db`, lanza su worker de inbox y el publicador de outbox en
background, y finalmente `serve` como proceso principal (ver
`containers/dev/entrypoints/*.sh`).

## Levantar el stack

Desde la raíz del repo:

```bash
# build + arranque en foreground (ves los logs de todos los servicios)
just dev-up

# o equivalente sin just:
podman-compose -f containers/dev/compose.yml up --build
```

Para arrancar en background:

```bash
just dev-up-detach
```

La primera vez tarda unos minutos: compila los 3 binarios Rust en
release, compila `quill-wasm` a WebAssembly y construye el bundle del
frontend con Vite. Las siguientes veces Docker/Podman cachea las capas
y es mucho más rápido si no cambiaste `Cargo.toml`/`package.json`.

## Verificar que todo está arriba

```bash
just health
```

Esto pega a `/health` de los tres servicios Rust y comprueba que `web`
responde 200. También puedes revisar contenedores y logs directamente:

```bash
just dev-ps              # estado de los contenedores
just dev-logs            # logs de todos los servicios
just dev-logs-svc web    # logs de un servicio puntual
```

## Probar el foro desde el navegador

Abre **http://localhost:8090**.

1. **Publicar** (`#new`): crea un usuario, una categoría, un tema, un
   post y un comentario, en ese orden — cada formulario necesita el
   `id` devuelto por el anterior (category_id, topic_id, post_id).
   Los campos de slug se autogeneran desde el título usando las
   funciones de `quill-wasm` (Rust compilado a WebAssembly, corriendo
   en el propio navegador).
2. **Buscar** (`#search`): una vez que el contenido se indexó (ver
   abajo), búscalo por texto libre — combina BM25 y similitud
   vectorial.

### Por qué la búsqueda puede tardar en aparecer

La creación de contenido es asíncrona: `content-service` escribe en su
outbox y lo publica a MQTT cuando corre `publish-outbox` (o el loop del
entrypoint, cada 2s en modo dev). `search-service` consume esos eventos
desde su inbox para indexar. En el stack de contenedores este ciclo es
automático — solo puede haber un par de segundos de latencia entre
crear un post y poder encontrarlo en el buscador.

## Probar el flujo end-to-end por línea de comandos

Como alternativa a la UI, puedes disparar el flujo completo con curl:

```bash
# 1. usuario
curl -s -X POST http://localhost:8080/users \
  -H 'Content-Type: application/json' \
  -d '{"username":"ada","email":"ada@example.com"}' | jq .

# 2. categoría
curl -s -X POST http://localhost:8081/categories \
  -H 'Content-Type: application/json' \
  -d '{"name":"Rust","slug":"rust"}' | jq .

# 3. tema (usa el id de categoría anterior)
curl -s -X POST http://localhost:8081/topics \
  -H 'Content-Type: application/json' \
  -d '{"category_id":"<CATEGORY_ID>","title":"Async runtimes","slug":"async-runtimes"}' | jq .

# 4. post (usa el id de tema anterior)
curl -s -X POST http://localhost:8081/posts \
  -H 'Content-Type: application/json' \
  -d '{"topic_id":"<TOPIC_ID>","title":"Tokio vs async-std","slug":"tokio-vs-async-std","body":"Comparativa de runtimes async en Rust..."}' | jq .

# 5. buscarlo (espera 1-2s a que se indexe)
just search "tokio"
# equivalente: curl -s "http://localhost:8082/search?q=tokio&limit=5" | jq .
```

También puedes inyectar eventos directamente al broker con los helpers
de `scripts/mqtt.just` (`just mqtt-create-post`, `just mqtt-watch`,
etc.) para inspeccionar el tráfico MQTT en vivo.

## Eventos en tiempo real (SSE)

`content-service` expone `GET /events` (Server-Sent Events) puenteando
los tópicos MQTT `forum.post.created` y `forum.comment.created`. El
frontend se suscribe automáticamente vía `EventSource`
(`web/src/api/events.ts`) — útil si quieres ver actualizaciones en
vivo sin recargar la página. Para inspeccionarlo manualmente:

```bash
curl -N http://localhost:8081/events
```

## Apagar y limpiar

```bash
just dev-down     # detiene contenedores, conserva los volúmenes (datos)
just dev-reset     # detiene y borra también los volúmenes (reset total)
```

## Reconstruir un solo servicio

Si solo cambiaste código de un servicio (p.ej. `content-service` o
`web`), no hace falta reconstruir todo el stack:

```bash
just dev-restart content-service
just dev-restart web
```

## Troubleshooting

- **Un servicio queda en `restart: on-failure` en loop**: revisa sus
  logs (`just dev-logs-svc <servicio>`). La causa más común es que
  Mosquitto todavía no pasó su healthcheck — los servicios Rust
  dependen de `mosquitto: condition: service_healthy` y deberían
  esperar automáticamente, pero si el broker tarda más de lo normal
  puede valer la pena reintentar con `just dev-restart <servicio>`.
- **El frontend carga pero las peticiones fallan con error de CORS o
  de red**: confirma que `users-service`, `content-service` y
  `search-service` están healthy (`just health`) y que no cambiaste
  los puertos en `containers/dev/compose.yml` sin actualizar también
  `web/src/api/client.ts` y `web/src/api/events.ts` (apuntan a
  `localhost:8080/8081/8082` por convención).
- **La búsqueda no devuelve nada**: confirma que el post se indexó —
  revisa `just dev-logs-svc search-service` buscando líneas de
  "indexed" o usa `just stats-search` para ver cuántos documentos
  tiene `search-service` en su base.
