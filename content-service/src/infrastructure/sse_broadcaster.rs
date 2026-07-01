use rumqttc::QoS;
use tokio::sync::broadcast;

use crate::infrastructure::mqtt;

const POST_CREATED_TOPIC: &str = "forum.post.created";
const COMMENT_CREATED_TOPIC: &str = "forum.comment.created";

/// Bridges MQTT content events into a broadcast channel that SSE clients
/// (browsers via EventSource) can subscribe to. Runs the blocking rumqttc
/// event loop on its own OS thread so it doesn't block the Tokio runtime.
pub fn spawn(host: String, port: u16) -> broadcast::Sender<String> {
    let (tx, _rx) = broadcast::channel(256);
    let tx_clone = tx.clone();

    std::thread::spawn(move || {
        let (client, mut connection) = mqtt::connect("content-service-sse", &host, port);

        for topic in [POST_CREATED_TOPIC, COMMENT_CREATED_TOPIC] {
            if let Err(error) = client.subscribe(topic, QoS::AtLeastOnce) {
                tracing::error!(topic, error = %error, "sse bridge failed to subscribe");
            }
        }

        for notification in connection.iter() {
            match notification {
                Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish))) => {
                    let payload = String::from_utf8_lossy(&publish.payload).to_string();
                    let event = serde_json::json!({
                        "type": publish.topic,
                        "payload": serde_json::from_str::<serde_json::Value>(&payload)
                            .unwrap_or(serde_json::Value::Null),
                    })
                    .to_string();
                    // Ignore send errors: no SSE clients connected is not an error.
                    let _ = tx_clone.send(event);
                }
                Ok(_) => {}
                Err(error) => {
                    tracing::error!(error = %error, "sse bridge mqtt connection error");
                    break;
                }
            }
        }
    });

    tx
}
