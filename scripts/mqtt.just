# MQTT recipes — imported by root Justfile
# Delegates to scripts/mqtt.sh for the actual mosquitto_pub/sub calls.
# Env vars: MQTT_HOST (default: localhost), MQTT_PORT (default: 1883)

# Subscribe to ALL quill topics (useful for end-to-end debugging)
[group('mqtt')]
mqtt-watch:
    scripts/mqtt.sh sub-all

# Subscribe to a single topic; press ^C to stop
# Usage: just mqtt-sub forum.post.created
[group('mqtt')]
mqtt-sub topic:
    scripts/mqtt.sh sub "{{topic}}"

# Publish a raw JSON payload to a topic
# Usage: just mqtt-pub forum.post.create.request '{"request_id":"...","topic_id":"...",...}'
[group('mqtt')]
mqtt-pub topic payload:
    scripts/mqtt.sh pub "{{topic}}" '{{payload}}'

# Create a post via MQTT (forum.post.create.request)
# Usage: just mqtt-create-post <topic_id> <title> <slug> <body>
[group('mqtt')]
mqtt-create-post topic_id title slug body:
    scripts/mqtt.sh create-post "{{topic_id}}" "{{title}}" "{{slug}}" "{{body}}"

# Create a comment via MQTT (forum.comment.create.request)
# Usage: just mqtt-create-comment <post_id> <body>
[group('mqtt')]
mqtt-create-comment post_id body:
    scripts/mqtt.sh create-comment "{{post_id}}" "{{body}}"

# Trigger a full search reindex via MQTT (forum.search.reindex.request)
[group('mqtt')]
mqtt-reindex:
    scripts/mqtt.sh trigger-reindex

# Subscribe to forum.deadletter to inspect failed messages
[group('mqtt')]
mqtt-deadletter:
    scripts/mqtt.sh sub forum.deadletter

# Request an embedding and wait for the response (round-trip smoke test)
# Usage: just mqtt-embedding "cars and driving"
[group('mqtt')]
mqtt-embedding text:
    scripts/mqtt.sh request-embedding "{{text}}"
