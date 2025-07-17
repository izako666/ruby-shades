use std::{error::Error, sync::Arc};

use axum::{Json, response::IntoResponse};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sled::{Db, IVec};

pub static DB: Lazy<Arc<Db>> =
    Lazy::new(|| Arc::new(sled::open("metadata").expect("failed to open sled db")));

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieMetadata {
    pub name: String,
    pub description: String,
    pub poster: String,
    pub backdrop: String,
}
#[derive(Debug, Serialize, Deserialize)]

pub struct TvShowMetadata {
    pub name: String,
    pub description: String,
    pub poster: String,
    pub backdrop: String,
    pub seasons: Vec<TvSeasonMetadata>,
}
#[derive(Debug, Serialize, Deserialize)]

pub struct TvSeasonMetadata {
    pub name: String,
    pub description: String,
    pub poster: String,
    pub episodes: Vec<TvEpisodeMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TvEpisodeMetadata {
    pub name: String,
    pub description: String,
    pub number: u16,
    pub poster: String,
}

pub enum Metadata {
    Show(TvShowMetadata),
    Movie(MovieMetadata),
    Episode(TvEpisodeMetadata),
}
impl IntoResponse for Metadata {
    fn into_response(self) -> axum::response::Response {
        match self {
            Metadata::Show(data) => {
                Json(serde_json::json!({ "type": "show", "data": data })).into_response()
            }
            Metadata::Movie(data) => {
                Json(serde_json::json!({ "type": "movie", "data": data })).into_response()
            }
            Metadata::Episode(data) => {
                Json(serde_json::json!({ "type": "episode", "data": data })).into_response()
            }
        }
    }
}

pub fn import_movie_metadata(
    local_path: &str,
    movie: MovieMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let byte_data = serde_json::to_vec(&movie)?;
    match DB.insert(local_path, byte_data) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn import_show_metadata(
    local_path: &str,
    show: TvShowMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let byte_data = serde_json::to_vec(&show)?;
    match DB.insert(local_path, byte_data) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn import_episode_metadata(
    local_path: &str,
    ep: TvEpisodeMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let byte_data = serde_json::to_vec(&ep)?;
    match DB.insert(local_path, byte_data) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn get_movie_metadata(local_path: &str) -> Result<MovieMetadata, Box<dyn std::error::Error>> {
    let vec_data = DB
        .get(local_path)
        .map_err(|e| Box::new(e) as Box<dyn Error>)?
        .ok_or_else(|| "Key not found")?;

    let movie = serde_json::from_slice::<MovieMetadata>(&vec_data)?;
    Ok(movie)
}
pub fn get_show_metadata(local_path: &str) -> Result<TvShowMetadata, Box<dyn std::error::Error>> {
    let vec_data = DB
        .get(local_path)
        .map_err(|e| Box::new(e) as Box<dyn Error>)?
        .ok_or_else(|| "Key not found")?;

    let show = serde_json::from_slice::<TvShowMetadata>(&vec_data)?;
    Ok(show)
}

pub fn get_episode_metadata(
    local_path: &str,
) -> Result<TvEpisodeMetadata, Box<dyn std::error::Error>> {
    let vec_data = DB
        .get(local_path)
        .map_err(|e| Box::new(e) as Box<dyn Error>)?
        .ok_or_else(|| "Key not found")?;

    let ep = serde_json::from_slice::<TvEpisodeMetadata>(&vec_data)?;
    Ok(ep)
}

pub fn get_metadata(local_path: &str) -> Result<Metadata, Box<dyn std::error::Error>> {
    if get_show_metadata(local_path).is_ok() {
        return Ok(Metadata::Show(get_show_metadata(local_path).unwrap()));
    } else if get_movie_metadata(local_path).is_ok() {
        return Ok(Metadata::Movie(get_movie_metadata(local_path).unwrap()));
    } else {
        return Ok(Metadata::Episode(get_episode_metadata(local_path).unwrap()));
    }
}

pub fn read_database() {
    println!("reading database");
    for entry in DB.iter() {
        match entry {
            Ok((key, value)) => {
                let key_str = String::from_utf8_lossy(&key);
                println!("🔑 Key: {}", key_str);
                // Try Show
                if let Ok(show) = serde_json::from_slice::<TvShowMetadata>(&value) {
                    println!("📺 TvShowMetadata:\n{:?}\n", show);
                    continue;
                }
                // Try Movie
                if let Ok(movie) = serde_json::from_slice::<MovieMetadata>(&value) {
                    println!("🎬 MovieMetadata:\n{:?}\n", movie);
                    continue;
                }

                // Try Episode
                if let Ok(episode) = serde_json::from_slice::<TvEpisodeMetadata>(&value) {
                    println!("📼 TvEpisodeMetadata:\n{:?}\n", episode);
                    continue;
                }

                // Unknown type
                println!("❓ Unknown format or corrupt data.\n");
            }
            Err(e) => {
                eprintln!("❌ Error reading entry: {}", e);
            }
        }
    }
}
