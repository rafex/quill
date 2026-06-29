mod adapters;
mod application;
mod domain;
mod infrastructure;
mod ports;
mod transport;

use std::sync::{Arc, Mutex};

use adapters::StubEmbeddingProvider;
use application::{HybridSearch, IndexContent};
use infrastructure::{db, inbox_worker, mqtt};
use ports::EmbeddingProvider;
use rumqttc::QoS;
use transport::AppState;

fn db_path() -> String {
    std::env::var("SEARCH_DB_PATH").unwrap_or_else(|_| "search.sqlite".to_string())
}

fn mqtt_host() -> String {
    std::env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".to_string())
}

fn mqtt_port() -> u16 {
    std::env::var("MQTT_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1883)
}

const DEADLETTER_TOPIC: &str = "forum.deadletter";

fn publish_to_deadletter(
    client: &rumqttc::Client,
    original_topic: &str,
    message_id: &str,
    payload: &str,
    error: &str,
) {
    let envelope = serde_json::json!({
        "original_topic": original_topic,
        "message_id": message_id,
        "error": error,
        "payload": payload,
    })
    .to_string();

    if let Err(e) = client.publish(DEADLETTER_TOPIC, QoS::AtLeastOnce, false, envelope) {
        eprintln!("failed to publish to {DEADLETTER_TOPIC}: {e}");
    }
}

#[cfg(feature = "onnx-embeddings")]
fn onnx_model_dir() -> String {
    std::env::var("SEARCH_ONNX_MODEL_DIR").unwrap_or_else(|_| "models/all-MiniLM-L6-v2".to_string())
}

/// Which EmbeddingProvider to wire up is an install-time/runtime choice
/// (SEARCH_EMBEDDING_PROVIDER=stub|onnx), not a hardcoded limitation: a
/// lightweight build can ship with only the deterministic stub (no ONNX
/// Runtime binary, no network at build time), while a build compiled with
/// `--features onnx-embeddings` can opt into real semantic search.
fn build_embedding_provider() -> Arc<dyn EmbeddingProvider> {
    let requested = std::env::var("SEARCH_EMBEDDING_PROVIDER").unwrap_or_else(|_| "stub".to_string());

    match requested.as_str() {
        "stub" => Arc::new(StubEmbeddingProvider),
        "onnx" => load_onnx_provider(),
        other => {
            eprintln!("unknown SEARCH_EMBEDDING_PROVIDER '{other}', falling back to stub");
            Arc::new(StubEmbeddingProvider)
        }
    }
}

#[cfg(feature = "onnx-embeddings")]
fn load_onnx_provider() -> Arc<dyn EmbeddingProvider> {
    let model_dir = onnx_model_dir();
    Arc::new(
        adapters::OnnxEmbeddingProvider::load(&model_dir).unwrap_or_else(|e| {
            panic!(
                "failed to load ONNX embedding provider from {model_dir}: {e}\n\
                 run `search-service download-model` first"
            )
        }),
    )
}

#[cfg(not(feature = "onnx-embeddings"))]
fn load_onnx_provider() -> Arc<dyn EmbeddingProvider> {
    panic!(
        "SEARCH_EMBEDDING_PROVIDER=onnx requires rebuilding with --features onnx-embeddings"
    )
}

fn main() {
    let command = std::env::args().nth(1).unwrap_or_else(|| "help".to_string());

    match command.as_str() {
        "init-db" => {
            let path = db_path();
            let conn = db::open(&path).expect("failed to open sqlite connection");
            db::init_schema(&conn).expect("failed to initialize schema");
            println!("initialized {path}");
        }
        "process-inbox" => process_inbox(),
        "serve" => serve(),
        "download-model" => download_model(),
        other => {
            eprintln!("unknown command: {other}");
            eprintln!("available commands: init-db, process-inbox, serve, download-model");
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "onnx-embeddings")]
fn download_model() {
    let model_dir = onnx_model_dir();
    adapters::download_model(&model_dir).expect("failed to download model");
}

#[cfg(not(feature = "onnx-embeddings"))]
fn download_model() {
    eprintln!("download-model requires rebuilding with --features onnx-embeddings");
    std::process::exit(1);
}

fn serve() {
    let conn = Arc::new(Mutex::new(open_db()));
    let embedding_provider = build_embedding_provider();
    let state = AppState {
        search: Arc::new(HybridSearch::new(conn, embedding_provider)),
    };

    let addr = std::env::var("SEARCH_HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8082".to_string());

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    runtime.block_on(async {
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .expect("failed to bind http listener");
        println!("search-service listening on {addr}");
        axum::serve(listener, transport::router(state))
            .await
            .expect("http server failed");
    });
}

fn open_db() -> rusqlite::Connection {
    let path = db_path();
    let conn = db::open(&path).expect("failed to open sqlite connection");
    db::init_schema(&conn).expect("failed to initialize schema");
    conn
}

fn process_inbox() {
    let conn = Arc::new(Mutex::new(open_db()));
    let embedding_provider = build_embedding_provider();
    let index_content = IndexContent::new(conn.clone(), embedding_provider);

    let (client, mut connection) =
        mqtt::connect("search-service-inbox", &mqtt_host(), mqtt_port());
    // MQTT wildcards ('+'/'#') only match whole levels separated by '/';
    // our topics use '.' as namespace separator, so there is no wildcard
    // that matches both - subscribe to each topic explicitly instead.
    let topics = std::env::var("SEARCH_INBOX_TOPICS").unwrap_or_else(|_| {
        "forum.post.created,forum.comment.created".to_string()
    });
    for topic in topics.split(',') {
        client
            .subscribe(topic, QoS::AtLeastOnce)
            .expect("failed to subscribe to inbox topic");
        println!("listening on {topic}");
    }

    for notification in connection.iter() {
        match notification {
            Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish))) => {
                let payload = String::from_utf8_lossy(&publish.payload).to_string();
                let parsed: Option<serde_json::Value> = serde_json::from_str(&payload).ok();

                let ext_id = parsed
                    .as_ref()
                    .and_then(|v| v.get("id"))
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                let body = parsed
                    .as_ref()
                    .and_then(|v| v.get("body"))
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                let title = parsed
                    .as_ref()
                    .and_then(|v| v.get("title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let content_type = if publish.topic.contains("comment") {
                    "comment"
                } else {
                    "post"
                };

                let (Some(ext_id), Some(body)) = (ext_id, body) else {
                    eprintln!("skipping malformed message on {}: {payload}", publish.topic);
                    continue;
                };

                let message_id = format!("{}:{}", publish.topic, ext_id);

                let result = inbox_worker::process_with_retry(
                    &conn,
                    &message_id,
                    &publish.topic,
                    &payload,
                    |_| {
                        index_content
                            .execute(&ext_id, content_type, &title, &body)
                            .map_err(|e: application::IndexContentError| e.to_string())
                    },
                );

                match result {
                    Ok(true) => println!("indexed {content_type} {ext_id}"),
                    Ok(false) => println!("skipped already-processed message {message_id}"),
                    Err(error) => {
                        eprintln!("giving up on message {message_id}: {error}");
                        publish_to_deadletter(&client, &publish.topic, &message_id, &payload, &error);
                    }
                }
            }
            Ok(_) => {}
            Err(error) => {
                eprintln!("mqtt connection error: {error}");
                break;
            }
        }
    }
}
