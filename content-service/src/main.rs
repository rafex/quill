mod adapters;
mod application;
mod domain;
mod infrastructure;
mod ports;
mod transport;

use std::sync::{Arc, Mutex};

use adapters::{
    MqttEventPublisher, SqliteCategoryRepository, SqliteCommentRepository, SqlitePostRepository,
    SqliteTopicRepository,
};
use application::{CreateComment, CreatePost, ReindexContent};
use infrastructure::{db, inbox_worker, mqtt, outbox_publisher};
use ports::{CommentRepository, PostRepository};
use rumqttc::QoS;
use transport::AppState;

const POST_CREATE_REQUEST_TOPIC: &str = "forum.post.create.request";
const COMMENT_CREATE_REQUEST_TOPIC: &str = "forum.comment.create.request";
const REINDEX_REQUEST_TOPIC: &str = "forum.search.reindex.request";
const DEADLETTER_TOPIC: &str = "forum.deadletter";

fn db_path() -> String {
    std::env::var("CONTENT_DB_PATH").unwrap_or_else(|_| "content.sqlite".to_string())
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

fn main() {
    let command = std::env::args().nth(1).unwrap_or_else(|| "help".to_string());

    match command.as_str() {
        "init-db" => {
            let path = db_path();
            let conn = db::open(&path).expect("failed to open sqlite connection");
            db::init_schema(&conn).expect("failed to initialize schema");
            println!("initialized {path}");
        }
        "serve" => serve(),
        "publish-outbox" => publish_outbox(),
        "process-inbox" => process_inbox(),
        "vacuum" => vacuum(),
        "stats" => stats(),
        "reindex" => reindex(),
        other => {
            eprintln!("unknown command: {other}");
            eprintln!("available commands: init-db, serve, publish-outbox, process-inbox, vacuum, stats, reindex");
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
    let conn = Arc::new(Mutex::new(open_db()));

    let state = AppState {
        categories: Arc::new(SqliteCategoryRepository::new(conn.clone())),
        topics: Arc::new(SqliteTopicRepository::new(conn.clone())),
        posts: Arc::new(SqlitePostRepository::new(conn.clone())),
        comments: Arc::new(SqliteCommentRepository::new(conn)),
    };

    let addr = std::env::var("CONTENT_HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8081".to_string());

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    runtime.block_on(async {
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .expect("failed to bind http listener");
        println!("content-service listening on {addr}");
        axum::serve(listener, transport::router(state))
            .await
            .expect("http server failed");
    });
}

fn publish_outbox() {
    let conn = Arc::new(Mutex::new(open_db()));

    let (client, mut connection) =
        mqtt::connect("content-service-outbox", &mqtt_host(), mqtt_port());

    std::thread::spawn(move || {
        for notification in connection.iter() {
            if notification.is_err() {
                break;
            }
        }
    });

    let publisher = MqttEventPublisher::new(client);
    let published = outbox_publisher::publish_pending(&conn, &publisher);
    // give the background event loop time to flush the publish over the network
    // before the process exits.
    std::thread::sleep(std::time::Duration::from_millis(300));
    println!("published {published} event(s)");
}

fn vacuum() {
    let path = db_path();
    let conn = db::open(&path).expect("failed to open sqlite connection");
    conn.execute_batch("VACUUM;").expect("VACUUM failed");
    println!("vacuumed {path}");
}

fn stats() {
    let conn = open_db();
    let categories: i64 = conn
        .query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))
        .unwrap_or(0);
    let topics: i64 = conn
        .query_row("SELECT COUNT(*) FROM topics", [], |r| r.get(0))
        .unwrap_or(0);
    let posts: i64 = conn
        .query_row("SELECT COUNT(*) FROM posts", [], |r| r.get(0))
        .unwrap_or(0);
    let comments: i64 = conn
        .query_row("SELECT COUNT(*) FROM comments", [], |r| r.get(0))
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

    println!("content-service stats ({}):", db_path());
    println!("  categories:      {categories}");
    println!("  topics:          {topics}");
    println!("  posts:           {posts}");
    println!("  comments:        {comments}");
    println!("  outbox pending:  {outbox_pending} / {outbox_total}");
    println!("  inbox processed: {inbox_processed}");
}

/// Republishes forum.post.created / forum.comment.created for all existing
/// content so search-service can reindex everything from scratch.
/// Equivalent to: ReindexContent::execute() + publish-outbox in one step.
fn reindex() {
    let conn = Arc::new(Mutex::new(open_db()));
    let posts: Arc<dyn PostRepository> = Arc::new(SqlitePostRepository::new(conn.clone()));
    let comments: Arc<dyn CommentRepository> = Arc::new(SqliteCommentRepository::new(conn.clone()));
    let reindex_content = ReindexContent::new(posts, comments);

    let (post_count, comment_count) = reindex_content
        .execute()
        .expect("failed to queue reindex events");
    println!("queued {post_count} post(s) and {comment_count} comment(s) for reindex");

    let (client, mut connection) =
        mqtt::connect("content-service-reindex", &mqtt_host(), mqtt_port());
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
    println!("published {published} event(s)");
}

fn process_inbox() {
    let conn = Arc::new(Mutex::new(open_db()));
    let posts: Arc<dyn PostRepository> = Arc::new(SqlitePostRepository::new(conn.clone()));
    let comments: Arc<dyn CommentRepository> = Arc::new(SqliteCommentRepository::new(conn.clone()));
    let create_post = CreatePost::new(posts.clone());
    let create_comment = CreateComment::new(comments.clone());
    let reindex_content = ReindexContent::new(posts, comments);

    let (client, mut connection) =
        mqtt::connect("content-service-inbox", &mqtt_host(), mqtt_port());

    for topic in [
        POST_CREATE_REQUEST_TOPIC,
        COMMENT_CREATE_REQUEST_TOPIC,
        REINDEX_REQUEST_TOPIC,
    ] {
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

                let request_id = parsed
                    .as_ref()
                    .and_then(|v| v.get("request_id"))
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| publish.pkid.to_string());
                let message_id = format!("{}:{}", publish.topic, request_id);

                let result = inbox_worker::process_with_retry(
                    &conn,
                    &message_id,
                    &publish.topic,
                    &payload,
                    |_| {
                        if publish.topic == REINDEX_REQUEST_TOPIC {
                            handle_reindex_request(&reindex_content)
                        } else {
                            handle_create_request(&publish.topic, &parsed, &create_post, &create_comment)
                        }
                    },
                );

                match result {
                    Ok(true) => println!("processed {message_id}"),
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

fn handle_reindex_request(reindex_content: &ReindexContent) -> Result<(), String> {
    reindex_content
        .execute()
        .map(|(posts, comments)| {
            println!("reindex queued {posts} post(s) and {comments} comment(s) for republishing; run publish-outbox to flush them");
        })
        .map_err(|e| format!("{e:?}"))
}

fn handle_create_request(
    topic: &str,
    payload: &Option<serde_json::Value>,
    create_post: &CreatePost,
    create_comment: &CreateComment,
) -> Result<(), String> {
    let payload = payload
        .as_ref()
        .ok_or_else(|| "payload is not valid JSON".to_string())?;

    let field = |name: &str| -> Result<String, String> {
        payload
            .get(name)
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| format!("missing or invalid field '{name}'"))
    };

    match topic {
        POST_CREATE_REQUEST_TOPIC => {
            let topic_id = field("topic_id")?;
            let title = field("title")?;
            let slug = field("slug")?;
            let body = field("body")?;
            create_post
                .execute(topic_id, title, slug, body)
                .map(|_| ())
                .map_err(|e| format!("{e:?}"))
        }
        COMMENT_CREATE_REQUEST_TOPIC => {
            let post_id = field("post_id")?;
            let body = field("body")?;
            create_comment
                .execute(post_id, body)
                .map(|_| ())
                .map_err(|e| format!("{e:?}"))
        }
        other => Err(format!("unexpected topic: {other}")),
    }
}
