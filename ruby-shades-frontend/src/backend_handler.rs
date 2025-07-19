use std::collections::HashMap;

use crate::config::read_config;
use anyhow::anyhow;
use futures_util::StreamExt;
use gloo_net::websocket::{Message, futures::WebSocket};
use reqwasm::http::{Request, Response};
use serde::{Deserialize, Serialize};
use yew::platform::spawn_local;

//We'll move all our structs from backend to here.

#[derive(Serialize, Deserialize, PartialEq)]
pub struct MetadataResponse {
    pub metadata: HashMap<String, Metadata>,
}
#[derive(Serialize, Deserialize, PartialEq)]
pub enum Metadata {
    Movie(MovieMetadata),
    Show(TvShowMetadata),
    Episode(TvEpisodeMetadata),
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct MovieMetadata {
    pub name: String,
    pub description: String,
    pub poster: String,
    pub backdrop: String,
}
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct TvShowMetadata {
    pub name: String,
    pub description: String,
    pub poster: String,
    pub backdrop: String,
    pub seasons: Vec<TvSeasonMetadata>,
}
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct TvSeasonMetadata {
    pub name: String,
    pub description: String,
    pub poster: String,
    pub episodes: Vec<TvEpisodeMetadata>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct TvEpisodeMetadata {
    pub name: String,
    pub description: String,
    pub number: u16,
    pub poster: String,
}

//Status update for websocket
#[derive(Deserialize, Serialize)]
pub struct StatusUpdate {
    progress: u8,
    status: String,
    uuid: String,
}

#[derive(Deserialize, Serialize)]
pub struct WatchResult {
    uuid: String,
    status: String,
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct PathObject {
    pub path: String,
    pub name: String,
    pub nested_paths: Vec<PathObject>,
}

pub async fn get_directory() -> Result<PathObject, anyhow::Error> {
    let backend_url = read_config().backend_url;

    let request = Request::get(&format!("{}/{}", &backend_url, "directory".to_string()))
        .send()
        .await;
    let response: Response = match request {
        Ok(resp) => resp,
        Err(e) => return Err(anyhow::anyhow!(e)),
    };

    if response.status() == 200 {
        let json = response
            .json::<PathObject>()
            .await
            .map_err(|e| anyhow!("Failed to parse JSON: {}", e))?;

        Ok(json)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        Err(anyhow!(
            "Request failed with status {}: {}",
            response.status(),
            error_text
        ))
    }
}

pub async fn get_metadata(resource: &str) -> Result<Metadata, anyhow::Error> {
    let backend_url = read_config().backend_url;
    let url = format!("{}/get_metadata?resource={}", backend_url, resource);

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to send request: {}", e))?;

    if response.status() == 200 {
        let metadata = response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| anyhow!("Failed to parse metadata JSON: {}", e))?;

        let nested_data = metadata.get("data").unwrap().clone();

        if let Ok(show) = serde_json::from_value::<TvShowMetadata>(nested_data.clone()) {
            return Ok(Metadata::Show(show));
        }
        if let Ok(movie) = serde_json::from_value::<MovieMetadata>(nested_data.clone()) {
            return Ok(Metadata::Movie(movie));
        }
        if let Ok(episode) = serde_json::from_value::<TvEpisodeMetadata>(nested_data) {
            return Ok(Metadata::Episode(episode));
        }

        Err(anyhow!("Unknown metadata format"))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        Err(anyhow!(
            "Request failed with status {}: {}",
            response.status(),
            error_text
        ))
    }
}

pub async fn start_watch(
    resource: &str,
    quality: Option<&str>,
) -> Result<WatchResult, anyhow::Error> {
    let backend_url = read_config().backend_url;
    let quality = quality.unwrap_or("1080p");

    let url = format!(
        "{}/watch?resource={}&quality={}",
        backend_url, resource, quality
    );

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to send request: {}", e))?;

    if response.status() == 200 {
        let result = response
            .json::<WatchResult>()
            .await
            .map_err(|e| anyhow!("Failed to parse WatchResult: {}", e))?;
        Ok(result)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        Err(anyhow!(
            "Request failed with status {}: {}",
            response.status(),
            error_text
        ))
    }
}

pub async fn listen_for_status_updates<F>(uuid: &str, mut callback: F) -> Result<(), anyhow::Error>
where
    F: FnMut(StatusUpdate) + 'static,
{
    let backend_url = read_config().backend_url.replace("http", "ws"); // converts http://localhost to ws://localhost
    let url = format!("{}/websocket_metadata?uuid={}", backend_url, uuid);

    let mut ws = WebSocket::open(&url).map_err(|e| anyhow!("WebSocket failed: {:?}", e))?;
    let (mut write, mut read) = ws.split();

    spawn_local(async move {
        while let Some(Ok(Message::Text(text))) = read.next().await {
            if let Ok(update) = serde_json::from_str::<StatusUpdate>(&text) {
                callback(update);
            } else {
                gloo::console::error!("Failed to parse StatusUpdate from WebSocket");
            }
        }
    });

    Ok(())
}

pub async fn get_all_metadata() -> Result<MetadataResponse, anyhow::Error> {
    let backend_url = read_config().backend_url;
    let url = format!("{}/get_all_metadata", backend_url);

    // Send GET request to the backend
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send request: {}", e))?;

    if response.status() == 200 {
        // Parse the JSON response body into MetadataResponse
        let metadata_response: MetadataResponse = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse metadata JSON: {}", e))?;

        Ok(metadata_response)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        Err(anyhow::anyhow!(
            "Request failed with status {}: {}",
            response.status(),
            error_text
        ))
    }
}
