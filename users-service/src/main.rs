mod adapters;
mod application;
mod domain;
mod infrastructure;
mod ports;
mod transport;

use std::sync::{Arc, Mutex};

use adapters::{MqttEventPublisher, SqliteUserRepository};
use infrastructure::{db, inbox_worker, mqtt, outbox_publisher};
use rumqttc::QoS;
use transport::AppState;
use tracing_subscriber::EnvFilter;

fn db_path() -> String {
    std::env::var("USERS_DB_PATH").unwrap_or_else(|_| "users.sqlite".to_string())
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

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}

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
        tracing::error!(topic = DEADLETTER_TOPIC, error = %e, "failed to publish to deadletter");
    }
}

fn main() {
    let command = std::env::args().nth(1).unwrap_or_else(|| "help".to_string());

    // stats/vacuum write to stdout directly; no tracing needed for those.
    match command.as_str() {
        "stats" => { stats(); return; }
        "vacuum" => { vacuum(); return; }
        _ => {}
    }

    init_tracing();

    match command.as_str() {
        "init-db" => {
            let path = db_path();
            let conn = db::open(&path).expect("failed to open sqlite connection");
            db::init_schema(&conn).expect("failed to initialize schema");
            tracing::info!(db = path, "database initialized");
        }
        "serve" => serve(),
        "publish-outbox" => publish_outbox(),
        "process-inbox" => process_inbox(),
        other => {
            eprintln!("unknown command: {other}");
            eprintln!("available commands: init-db, serve, publish-outbox, process-inbox, vacuum, stats");
            std::process::exit(1);
        }
    }
}

fn open_db() -> rusqlite::Connection {
    let path = db_path();
    let conn = db::open(&path).expect("failed to open sqlite connection");
    db::init_schema(&conn).expect("failed to initialize schema");
    conn
}

fn serve() {
    let conn = open_db();
    let repo: Arc<dyn ports::UserRepository> =
        Arc::new(SqliteUserRepository::new(Arc::new(Mutex::new(conn))));
    let state = AppState { repo };

    let addr = std::env::var("USERS_HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    runtime.block_on(async {
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .expect("failed to bind http listener");
        tracing::info!(addr, "users-service listening");
        axum::serve(listener, transport::router(state))
            .await
            .expect("http server failed");
    });
}

fn publish_outbox() {
    let conn = Arc::new(Mutex::new(open_db()));

    let (client, mut connection) = mqtt::connect("users-service-outbox", &mqtt_host(), mqtt_port());

    std::thread::spawn(move || {
        for notification in connection.iter() {
            if notification.is_err() {
                break;
            }
        }
    });

    let publisher = MqttEventPublisher::new(client);
    let published = outbox_publisher::publish_pending(&conn, &publisher);
    std::thread::sleep(std::time::Duration::from_millis(300));
    tracing::info!(published, "outbox flushed");
}

fn vacuum() {
    let path = db_path();
    let conn = db::open(&path).expect("failed to open sqlite connection");
    conn.execute_batch("VACUUM;").expect("VACUUM failed");
    println!("vacuumed {path}");
}

fn stats() {
    let conn = open_db();
    let users: i64 = conn
        .query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))
        .unwrap_or(0);
    let outbox_pending: i64 = conn
        .query_row("SELECT COUNT(*) FROM outbox_events WHERE published_at IS NULL", [], |r| r.get(0))
        .unwrap_or(0);
    let outbox_total: i64 = conn
        .query_row("SELECT COUNT(*) FROM outbox_events", [], |r| r.get(0))
        .unwrap_or(0);
    let inbox_processed: i64 = conn
        .query_row("SELECT COUNT(*) FROM inbox_messages", [], |r| r.get(0))
        .unwrap_or(0);

    println!("users-service stats ({}):", db_path());
    println!("  users:           {users}");
    println!("  outbox pending:  {outbox_pending} / {outbox_total}");
    println!("  inbox processed: {inbox_processed}");
}

fn process_inbox() {
    let conn = Arc::new(Mutex::new(open_db()));

    let (client, mut connection) = mqtt::connect("users-service-inbox", &mqtt_host(), mqtt_port());
    let topic = std::env::var("USERS_INBOX_TOPIC").unwrap_or_else(|_| "forum.user.command".to_string());
    client
        .subscribe(&topic, QoS::AtLeastOnce)
        .expect("failed to subscribe to inbox topic");
    tracing::info!(topic, "inbox listening");

    for notification in connection.iter() {
        match notification {
            Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish))) => {
                let payload = String::from_utf8_lossy(&publish.payload).to_string();
                let message_id = serde_json::from_str::<serde_json::Value>(&payload)
                    .ok()
                    .and_then(|v| v.get("message_id").and_then(|m| m.as_str().map(str::to_string)))
                    .unwrap_or_else(|| publish.pkid.to_string());

                tracing::debug!(message_id, topic = publish.topic, "received message");

                let result = inbox_worker::process_with_retry(
                    &conn,
                    &message_id,
                    &publish.topic,
                    &payload,
                    |payload| {
                        tracing::debug!(message_id, "processing inbox message");
                        let _ = payload;
                        Ok(())
                    },
                );

                match result {
                    Ok(true) => tracing::info!(message_id, "processed"),
                    Ok(false) => tracing::debug!(message_id, "skipped already-processed message"),
                    Err(ref error) => {
                        tracing::error!(message_id, error, "giving up, sending to deadletter");
                        publish_to_deadletter(&client, &publish.topic, &message_id, &payload, error);
                    }
                }
            }
            Ok(_) => {}
            Err(ref error) => {
                tracing::error!(error = %error, "mqtt connection error");
                break;
            }
        }
    }
}
