#!/usr/bin/env bash
# mqtt.sh — thin wrapper around mosquitto_pub / mosquitto_sub
# Called by mqtt.just recipes; can also be used directly from the shell.
#
# Usage:
#   scripts/mqtt.sh pub  <topic> <json-payload>
#   scripts/mqtt.sh sub  <topic> [count]
#   scripts/mqtt.sh sub-all            # subscribe to all quill topics
#   scripts/mqtt.sh create-post  <topic_id> <title> <slug> <body>
#   scripts/mqtt.sh create-comment <post_id> <body>
#   scripts/mqtt.sh trigger-reindex
#   scripts/mqtt.sh request-embedding <text>

set -euo pipefail

MQTT_HOST="${MQTT_HOST:-localhost}"
MQTT_PORT="${MQTT_PORT:-1883}"

_pub() {
    mosquitto_pub -h "$MQTT_HOST" -p "$MQTT_PORT" -t "$1" -m "$2" -q 1
}

_sub() {
    local topic="$1"
    local count="${2:-0}"    # 0 = indefinite
    if [[ "$count" -gt 0 ]]; then
        mosquitto_sub -h "$MQTT_HOST" -p "$MQTT_PORT" -t "$topic" -C "$count" -v
    else
        mosquitto_sub -h "$MQTT_HOST" -p "$MQTT_PORT" -t "$topic" -v
    fi
}

cmd="${1:-help}"
shift || true

case "$cmd" in
    pub)
        topic="$1"; payload="$2"
        echo "→ $topic  $payload"
        _pub "$topic" "$payload"
        ;;

    sub)
        topic="${1:-#}"; count="${2:-0}"
        echo "← subscribing to $topic (^C to stop)"
        _sub "$topic" "$count"
        ;;

    sub-all)
        echo "← subscribing to all quill topics (^C to stop)"
        mosquitto_sub -h "$MQTT_HOST" -p "$MQTT_PORT" \
            -t "forum.post.created" \
            -t "forum.comment.created" \
            -t "forum.post.create.request" \
            -t "forum.comment.create.request" \
            -t "forum.search.reindex.request" \
            -t "forum.embedding.generate.request" \
            -t "forum.embedding.generated" \
            -t "forum.deadletter" \
            -v
        ;;

    create-post)
        topic_id="${1:?topic_id required}"
        title="${2:?title required}"
        slug="${3:?slug required}"
        body="${4:?body required}"
        request_id="$(uuidgen | tr '[:upper:]' '[:lower:]')"
        payload=$(printf '{"request_id":"%s","topic_id":"%s","title":"%s","slug":"%s","body":"%s"}' \
            "$request_id" "$topic_id" "$title" "$slug" "$body")
        echo "→ forum.post.create.request  request_id=$request_id"
        _pub "forum.post.create.request" "$payload"
        ;;

    create-comment)
        post_id="${1:?post_id required}"
        body="${2:?body required}"
        request_id="$(uuidgen | tr '[:upper:]' '[:lower:]')"
        payload=$(printf '{"request_id":"%s","post_id":"%s","body":"%s"}' \
            "$request_id" "$post_id" "$body")
        echo "→ forum.comment.create.request  request_id=$request_id"
        _pub "forum.comment.create.request" "$payload"
        ;;

    trigger-reindex)
        payload='{"triggered_by":"cli"}'
        echo "→ forum.search.reindex.request"
        _pub "forum.search.reindex.request" "$payload"
        ;;

    request-embedding)
        text="${1:?text required}"
        request_id="$(uuidgen | tr '[:upper:]' '[:lower:]')"
        payload=$(printf '{"request_id":"%s","text":"%s"}' "$request_id" "$text")
        echo "→ forum.embedding.generate.request  request_id=$request_id"
        _pub "forum.embedding.generate.request" "$payload"
        echo "← waiting for forum.embedding.generated (1 message)..."
        mosquitto_sub -h "$MQTT_HOST" -p "$MQTT_PORT" \
            -t "forum.embedding.generated" -C 1 -v
        ;;

    help|*)
        echo "Usage: scripts/mqtt.sh <command> [args]"
        echo ""
        echo "Commands:"
        echo "  pub <topic> <json>                     publish a raw JSON payload"
        echo "  sub <topic> [count]                    subscribe (0=indefinite)"
        echo "  sub-all                                subscribe to all quill topics"
        echo "  create-post <topic_id> <title> <slug> <body>"
        echo "  create-comment <post_id> <body>"
        echo "  trigger-reindex                        send forum.search.reindex.request"
        echo "  request-embedding <text>               send + wait for embedding response"
        echo ""
        echo "Env: MQTT_HOST (default: localhost)  MQTT_PORT (default: 1883)"
        ;;
esac
