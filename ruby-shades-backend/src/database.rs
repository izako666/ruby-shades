use std::{error::Error, sync::Arc};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sled::{Db, IVec};

static DB: Lazy<Arc<Db>> =
    Lazy::new(|| Arc::new(sled::open("metadata").expect("failed to open sled db")));

#[derive(Serialize, Deserialize)]
pub struct MovieMetadata {
    name: String,
    description: String,
    poster: String,
    backdrop: String,
}
#[derive(Serialize, Deserialize)]

pub struct TvShowMetadata {
    name: String,
    description: String,
    poster: String,
    backdrop: String,
    seasons: Vec<TvSeasonMetadata>,
}
#[derive(Serialize, Deserialize)]

pub struct TvSeasonMetadata {
    name: String,
    description: String,
    poster: String,
    episodes: Vec<TvEpisodeMetadata>,
}

#[derive(Serialize, Deserialize)]
pub struct TvEpisodeMetadata {
    name: String,
    description: String,
    number: u16,
    poster: String,
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
