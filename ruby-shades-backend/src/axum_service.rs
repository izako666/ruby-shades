use std::{
    collections::HashMap,
    fs,
    sync::{Arc, Mutex},
};

use crate::{
    config::{Config, read_config},
    directory_parser::{self, PathObject},
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
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
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

//List of clients id's mapped with their users, Mutex for multi-thread nature of websockets.
type Clients = Lazy<Arc<Mutex<HashMap<String, Option<Vec<UnboundedSender<Message>>>>>>>;
static CLIENTS: Clients = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

//Status update for websocket
#[derive(Debug, Deserialize, Serialize)]
struct StatusUpdate {
    progress: u8,
    status: String,
    uuid: String,
}
//Possible errors for websocket
enum ServiceErrors {
    BadRequestNoResource,
    UnknownInternalServer,
}

//quality mapping bitrate and quality options, might be inaccurate
static BITRATE_QUALITY_MAP: &[(&str, u32)] = &[
    ("1440p", 8000),
    ("1080p", 6000),
    ("720p", 3000),
    ("480p", 1500),
    ("360p", 1000),
    ("240p", 500),
];

//initializes REST endpoints, websocket endpoints, and serves the server
pub async fn initialize() {
    let app = Router::new()
        .route("/", get(|| async { "Ruby Shades Backend" }))
        .nest_service("/videos", get_service(ServeDir::new("static/videos")))
        .route("/watch", get(handle_watch))
        .route("/websocket_metadata", any(ws_handler))
        .route("/directory", get(handle_directory));
    let config: Config = read_config();
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.address, config.port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

//returns bitrate from quality
fn get_bitrate_from_quality(quality: &str) -> Option<u32> {
    for &(qual, bitrate) in BITRATE_QUALITY_MAP {
        if quality == qual {
            return Some(bitrate);
        }
    }
    None
}

pub async fn convert_video_to_hls(
    input_path: &str,
    output_dir: &str,
    quality: &str,
) -> std::io::Result<()> {
    fs::create_dir_all(output_dir)?; // Ensure output dir exists
    let quality_valid = quality.replace("p", "");

    let bitrate: u32 = get_bitrate_from_quality(quality).unwrap_or(400);
    let status = Command::new("ffmpeg")
        .args(&[
            "-i",
            input_path,
            "-c:v",
            "libx264",
            "-profile:v",
            "baseline",
            "-vf",
            &format!("scale=trunc(iw*{}/ih/2)*2:{}", quality_valid, quality_valid),
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

//mapped to watch endpoint takes resource query parameter, finds video from it, starts transcoding video to hls
async fn handle_watch(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, ServiceErrors> {
    let path = params.get("resource");
    let default_quality = String::from("1080p");
    let quality = params.get("quality").unwrap_or(&default_quality);

    if let Some(path) = path {
        let path_str: String = String::from(path);
        let uuid_new: Uuid = Uuid::new_v4();
        let quality_clone = quality.clone();
        let output_dir = format!("static/videos/{}/", uuid_new.clone());
        println!("task spawning");
        task::spawn(async move {
            if let Err(e) = convert_video_to_hls(&path_str, &output_dir, &quality_clone).await {
                // Log the error, mark failure somewhere
                println!("FAILURE: {e}");

                notify_clients(
                    &uuid_new.to_string(),
                    &StatusUpdate {
                        progress: 0,
                        status: String::from("FAILURE"),
                        uuid: uuid_new.to_string(),
                    },
                );
            } else {
                println!("success");
                notify_clients(
                    &uuid_new.to_string(),
                    &StatusUpdate {
                        progress: 100,
                        status: String::from("SUCCESS"),
                        uuid: uuid_new.to_string(),
                    },
                );
            }
        });

        CLIENTS.lock().unwrap().insert(uuid_new.to_string(), None);
        Ok(Json(json!({
            "uuid": uuid_new.to_string(),
            "status": "processing"
        })))
    } else {
        println!("Error bad request");
        Err(ServiceErrors::BadRequestNoResource)
    }
}

//mapping service errors to error responses
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

//upgrades http connection to ws connection
async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, ServiceErrors> {
    let uuid = params.get("uuid");
    if let None = uuid {
        return Err(ServiceErrors::BadRequestNoResource);
    }
    let uuid_clone = uuid.unwrap().clone();

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, uuid_clone.to_string())))
}

//takes user connection, creates a uuid and maps that uuid with a new channel sender, so that we can send messages to that user.
async fn handle_socket(socket: WebSocket, uuid: String) {
    let (sender, _receiver) = socket.split();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
    CLIENTS
        .lock()
        .unwrap()
        .entry(uuid.to_string())
        .or_default()
        .clone()
        .unwrap_or(Vec::new())
        .push(tx.clone());

    if !CLIENTS.lock().unwrap().contains_key(&uuid) {
        CLIENTS.lock().unwrap().insert(uuid, Some(vec![tx.clone()]));
    } else {
        let mut clients = CLIENTS.lock().unwrap();

        if let Some(Some(vec)) = clients.get_mut(&uuid) {
            vec.push(tx.clone());
        }
    }
    tokio::spawn(write(sender, rx));
    //tokio::spawn(read(receiver, tx, uuid));
}
// async fn read(mut receiver: SplitStream<WebSocket>, tx: UnboundedSender<Message>, uuid: Uuid) {
//     // while let Some(Ok(Message::Close(text))) = receiver.next().await {
//     //     if CLIENTS.lock().unwrap().contains_key(&uuid.to_string()) {
//     //         CLIENTS.lock().unwrap().remove(&uuid.to_string());
//     //     }
//     // }
// }

async fn write(mut sender: SplitSink<WebSocket, Message>, mut rx: UnboundedReceiver<Message>) {
    while let Some(msg) = rx.recv().await {
        if sender.send(msg).await.is_err() {
            break;
        }
    }
}

//helper function to send message to clients
fn notify_clients(uuid: &str, payload: &StatusUpdate) {
    let mut clients_map = CLIENTS.lock().unwrap();
    let payload_str = serde_json::to_string(payload);
    if let Ok(payload_str) = payload_str {
        let payload_bytes = Utf8Bytes::from(payload_str);
        if let Some(subscribers) = clients_map.get_mut(uuid) {
            if subscribers.is_some() {
                subscribers
                    .as_mut()
                    .unwrap()
                    .retain(|tx| tx.send(Message::Text(payload_bytes.clone())).is_ok());
            }
        }
    }
}

async fn handle_directory() -> Result<Json<PathObject>, ServiceErrors> {
    let path_obj = directory_parser::PATH_OBJECT.lock().unwrap();
    if let Some(path) = path_obj.clone() {
        return Ok(Json(path));
    } else {
        return Err(ServiceErrors::UnknownInternalServer);
    }
}
