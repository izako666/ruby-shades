use std::{
    collections::HashMap,
    fs,
    sync::{Arc, Mutex},
};

use axum::{
    Json, Router,
    extract::{
        Query, WebSocketUpgrade,
        ws::{Message, Utf8Bytes, WebSocket},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{any, get, get_service},
};
use ffmpeg_next as ffmpeg;
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::{
    process::Command,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task,
};
use tower_http::services::ServeDir;
use uuid::Uuid;

type Clients = Lazy<Arc<Mutex<HashMap<String, Vec<UnboundedSender<Message>>>>>>;
static CLIENTS: Clients = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));
#[derive(Debug, Deserialize, Serialize)]
struct StatusUpdate {
    progress: u8,
    status: String,
}
enum ServiceErrors {
    BadRequestNoResource,
    UnknownInternalServer,
}

use crate::config::{Config, read_config};
static BITRATE_QUALITY_MAP: &[(&str, u32)] = &[
    ("1440p", 4000),
    ("1080p", 4000),
    ("720p", 3000),
    ("480p", 1200),
    ("360p", 700),
    ("240p", 400),
];
pub async fn initialize() {
    let app = Router::new()
        .route("/", get(|| async { "Ruby Shades Backend" }))
        .nest_service("/videos", get_service(ServeDir::new("/static/videos")))
        .route("/watch", get(handle_watch))
        .route("/websocket_metadata", any(ws_handler));
    init_ffmpeg();
    let config: Config = read_config();
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.address, config.port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
fn get_quality(quality: &str) -> Option<u32> {
    for &(qual, bitrate) in BITRATE_QUALITY_MAP {
        if quality == qual {
            return Some(bitrate);
        }
    }
    None
}
fn init_ffmpeg() -> anyhow::Result<()> {
    ffmpeg::init()?;
    Ok(())
}
pub async fn convert_video_to_hls(
    input_path: &str,
    output_dir: &str,
    quality: &str,
) -> std::io::Result<()> {
    fs::create_dir_all(output_dir)?; // Ensure output dir exists
    let quality_valid = quality.replace("p", "");

    let bitrate: u32 = get_quality(quality).unwrap_or(400);
    let status = Command::new("ffmpeg")
        .args(&[
            "-i",
            input_path,
            "-profile:v",
            "baseline",
            "-vf",
            format!("scale=-1:{}", quality_valid).as_str(),
            "-level",
            "3.0",
            "-start_number",
            "0",
            "-hls_time",
            "12",
            "-hls_list_size",
            "0",
            "-b:v",
            &format!("{}k", bitrate),
            "-f",
            "hls",
            &format!("{}/index.m3u8", output_dir),
        ])
        .status()
        .await;

    if !status.is_ok_and(|f| f.success()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ffmpeg failed",
        ));
    }

    Ok(())
}

async fn handle_watch(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, ServiceErrors> {
    let path = params.get("resource");

    if let Some(path) = path {
        let path_str: String = String::from(path);
        let uuid_new: Uuid = Uuid::new_v4();
        let output_dir = format!("/static/videos/{}/", uuid_new.clone());
        task::spawn(async move {
            if let Err(e) = convert_video_to_hls(&path_str, &output_dir, "1080p").await {
                // Log the error, mark failure somewhere
                eprintln!("Failed to convert video: {}", e);
                // Optional: Write status to disk/db/etc
            } else {
                // Optional: write a "done" file to signal job completion
            }
        });

        Ok(Json(json!({
            "uuid": uuid_new.to_string(),
            "status": "processing"
        })))
    } else {
        Err(ServiceErrors::BadRequestNoResource)
    }
}

impl IntoResponse for ServiceErrors {
    fn into_response(self) -> Response {
        let body = match self {
            ServiceErrors::BadRequestNoResource => {
                (StatusCode::BAD_REQUEST, "something went wrong")
            }
            ServiceErrors::UnknownInternalServer => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Something bad happened")
            }
        };

        body.into_response()
    }
}
async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}
async fn handle_socket(socket: WebSocket) {
    let (sender, receiver) = socket.split();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
    let uuid_new = Uuid::new_v4();
    CLIENTS
        .lock()
        .unwrap()
        .entry(uuid_new.to_string())
        .or_default()
        .push(tx.clone());
    tokio::spawn(write(sender, rx));
    tokio::spawn(read(receiver, tx, uuid_new));
}
async fn read(mut receiver: SplitStream<WebSocket>, tx: UnboundedSender<Message>, uuid: Uuid) {
    while let Some(Ok(Message::Close(text))) = receiver.next().await {
        if CLIENTS.lock().unwrap().contains_key(&uuid.to_string()) {
            CLIENTS.lock().unwrap().remove(&uuid.to_string());
        }
    }
}

async fn write(mut sender: SplitSink<WebSocket, Message>, mut rx: UnboundedReceiver<Message>) {
    while let Some(msg) = rx.recv().await {
        if sender.send(msg).await.is_err() {
            break;
        }
    }
}

fn notify_clients(uuid: &str, payload: &StatusUpdate) {
    let mut clients_map = CLIENTS.lock().unwrap();
    let payload_str = serde_json::to_string(payload);
    if let Ok(payload_str) = payload_str {
        let payload_bytes = Utf8Bytes::from(payload_str);
        if let Some(subscribers) = clients_map.get_mut(uuid) {
            subscribers.retain(|tx| tx.send(Message::Text(payload_bytes.clone())).is_ok());
        }
    }
}
