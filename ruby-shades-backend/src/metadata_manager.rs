use std::{error::Error, path::Path, pin::Pin};

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;

use crate::{
    config::read_config,
    database::{self, MovieMetadata, TvEpisodeMetadata, TvSeasonMetadata, TvShowMetadata},
    directory_parser::{self, PathObject},
};

#[derive(Debug, Deserialize)]
pub struct SearchResponses {
    results: Vec<SearchResult>,
}
#[derive(Debug, Deserialize)]
pub struct TvSearchResponses {
    results: Vec<TvSearchResult>,
}
#[derive(Debug, Deserialize)]
pub struct SearchResult {
    id: u32,
    backdrop_path: Option<String>,
    poster_path: Option<String>,
    title: String,
}

#[derive(Debug, Deserialize)]
pub struct TvSearchResult {
    id: u32,
    backdrop_path: Option<String>,
    poster_path: Option<String>,
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct TvDetails {
    pub backdrop_path: Option<String>,
    pub name: String,
    pub number_of_seasons: u32,
    pub overview: String,
    pub poster_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Episode {
    pub name: String,
    pub overview: String,
    pub episode_number: u16,
    pub still_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SeasonDetails {
    pub name: String,
    pub poster_path: Option<String>,
    pub overview: String,
    pub season_number: u16,
    pub episodes: Vec<Episode>,
}

#[derive(Debug, Deserialize)]
pub struct MovieDetails {
    pub title: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
}

pub async fn search_tmdb_movie(
    query: &str,
) -> Result<Option<SearchResult>, Box<dyn std::error::Error>> {
    let token = read_config().tmdb_auth_token;

    let client = reqwest::Client::new();

    let response = client
        .get("https://api.themoviedb.org/3/search/movie")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header(CONTENT_TYPE, "application/json;charset=utf-8")
        .query(&[("query", query), ("include_adult", "true")])
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("TMDB request failed with status: {}", response.status()).into());
    }

    let search_results: SearchResponses = response.json().await?;

    Ok(search_results.results.into_iter().next())
}
pub async fn search_tmdb_tv(
    query: &str,
) -> Result<Option<TvSearchResult>, Box<dyn std::error::Error>> {
    let token = read_config().tmdb_auth_token;

    let client = reqwest::Client::new();

    let response = client
        .get("https://api.themoviedb.org/3/search/tv")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header(CONTENT_TYPE, "application/json;charset=utf-8")
        .query(&[("query", query), ("include_adult", "true")])
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("TMDB TV search failed: {}", response.status()).into());
    }

    let search_results: TvSearchResponses = response.json().await?;
    Ok(search_results.results.into_iter().next())
}

pub async fn fetch_tv_series_details(
    series_id: u32,
) -> Result<TvDetails, Box<dyn std::error::Error>> {
    let token = read_config().tmdb_auth_token;

    let client = reqwest::Client::new();
    let url = format!("https://api.themoviedb.org/3/tv/{}", series_id);

    let response = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header(CONTENT_TYPE, "application/json;charset=utf-8")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!(
            "TMDB TV series details request failed: {}",
            response.status()
        )
        .into());
    }

    let details: TvDetails = response.json().await?;
    Ok(details)
}

pub async fn fetch_tv_season_details(
    series_id: u32,
    season_number: u32,
) -> Result<SeasonDetails, Box<dyn Error>> {
    let token = read_config().tmdb_auth_token;

    let client = reqwest::Client::new();
    let url = format!(
        "https://api.themoviedb.org/3/tv/{}/season/{}",
        series_id, season_number
    );

    let response = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header(CONTENT_TYPE, "application/json;charset=utf-8")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!(
            "TMDB TV season details request failed: {}",
            response.status()
        )
        .into());
    }

    let details: SeasonDetails = response.json().await?;
    Ok(details)
}

pub async fn fetch_movie_details(movie_id: u32) -> Result<MovieDetails, Box<dyn Error>> {
    let token = read_config().tmdb_auth_token;

    let client = reqwest::Client::new();
    let url = format!("https://api.themoviedb.org/3/movie/{}", movie_id);

    let response = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header(CONTENT_TYPE, "application/json;charset=utf-8")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("TMDB movie details request failed: {}", response.status()).into());
    }

    let details: MovieDetails = response.json().await?;
    Ok(details)
}

pub async fn transcode_path_object(
    path_obj: &PathObject,
    season_number: Option<u16>,
) -> Result<(), anyhow::Error> {
    let cleaned_path_name = path_obj.name.clone().replace(&['_', '.', '-'][..], " ");
    let is_file = path_obj.nested_paths.is_empty()
        && directory_parser::is_video_file(Path::new(&path_obj.path));

    if is_file {
        let is_episode = is_episode_name(&cleaned_path_name);
        if is_episode {
            let episode_and_season_number = extract_season_and_episode(&cleaned_path_name);
            if let Some(episode_and_season_number) = episode_and_season_number {
                let season_number_extracted = episode_and_season_number
                    .0
                    .unwrap_or(season_number.unwrap_or(1).into());

                let query_name = extract_clean_name(&path_obj.name);
                let search_obj = search_tmdb_tv(&query_name).await;
                if let Ok(Some(search_obj)) = search_obj {
                    let season_details =
                        fetch_tv_season_details(search_obj.id, season_number_extracted.into())
                            .await;
                    if let Ok(season_details) = season_details {
                        let episode_details = season_details
                            .episodes
                            .get(episode_and_season_number.1 as usize - 1);
                        if let Some(episode_details) = episode_details {
                            let _ = database::import_episode_metadata(
                                &path_obj.path,
                                TvEpisodeMetadata {
                                    name: episode_details.name.clone(),
                                    description: episode_details.overview.clone(),
                                    number: episode_details.episode_number,
                                    poster: episode_details.still_path.clone().unwrap_or_default(),
                                },
                            );
                        }
                    }
                }
            }
        } else {
            let query_movie_name = extract_clean_name(&path_obj.name);
            let search_movie = search_tmdb_movie(&query_movie_name).await;
            if let Ok(Some(search_movie)) = search_movie {
                let movie_details = fetch_movie_details(search_movie.id).await;
                if let Ok(movie_details) = movie_details {
                    let _ = database::import_movie_metadata(
                        &path_obj.path,
                        MovieMetadata {
                            name: movie_details.title,
                            description: movie_details.overview.unwrap_or_default(),
                            poster: movie_details.poster_path.unwrap_or_default(),
                            backdrop: movie_details.backdrop_path.unwrap_or_default(),
                        },
                    );
                }
            }
        }
    } else {
        for path in &path_obj.nested_paths {
            let season_number = extract_season_number(&path.name);
            if let Some(_season_num) = season_number {
                //we should get the parent path name
                let potential_parent = Path::new(&path.path).parent();
                if let Some(potential_parent) = potential_parent {
                    let file_name = potential_parent.file_name();
                    if let Some(file_name) = file_name {
                        let file_name_str = file_name.to_str();
                        if let Some(file_name_str) = file_name_str {
                            let show_query = extract_clean_name(&file_name_str);
                            let show_search = search_tmdb_tv(&show_query).await;
                            if let Ok(Some(show_search)) = show_search {
                                let show_details = fetch_tv_series_details(show_search.id).await;
                                if let Ok(show_details) = show_details {
                                    let mut seasons: Vec<TvSeasonMetadata> = Vec::new();
                                    for i in 0..show_details.number_of_seasons as usize {
                                        let season_result = fetch_tv_season_details(
                                            show_search.id,
                                            (i + 1).try_into().unwrap(),
                                        )
                                        .await;

                                        if let Ok(season_result) = season_result {
                                            seasons.push(TvSeasonMetadata {
                                                name: season_result.name,
                                                description: season_result.overview,
                                                poster: season_result
                                                    .poster_path
                                                    .unwrap_or_default(),
                                                episodes: season_result
                                                    .episodes
                                                    .iter()
                                                    .map(|ep| TvEpisodeMetadata {
                                                        name: ep.name.clone(),
                                                        description: ep.overview.clone(),
                                                        number: ep.episode_number,
                                                        poster: ep
                                                            .still_path
                                                            .clone()
                                                            .unwrap_or_default(),
                                                    })
                                                    .collect(),
                                            });
                                        }
                                    }
                                    //i see no reason why to_str would fail this late in the game lol

                                    let _ = database::import_show_metadata(
                                        potential_parent.to_str().unwrap(),
                                        TvShowMetadata {
                                            name: show_details.name,
                                            description: show_details.overview,
                                            poster: show_details.poster_path,
                                            backdrop: show_details
                                                .backdrop_path
                                                .unwrap_or_default(),
                                            seasons,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
            let current_season_num = extract_season_number(&path_obj.name);
            let fut = transcode_path_object(&path, current_season_num);
            let _ = Pin::from(Box::new(fut)).await;
        }
    }
    Ok(())
}

fn is_episode_name(name: &str) -> bool {
    let re = regex::Regex::new(
        r"(?i)(s\d{1,2}e\d{1,2}|season\d{1,2}episode\d{1,2}|e\d{1,2}|episode\d{1,2})",
    )
    .unwrap();
    re.is_match(name)
}

fn extract_season_number(name: &str) -> Option<u16> {
    let re = regex::Regex::new(r"(?i)(?:s|season)(\d{1,2})").unwrap();
    re.captures(name)
        .and_then(|caps| caps.get(1)?.as_str().parse::<u16>().ok())
}

fn extract_season_and_episode(name: &str) -> Option<(Option<u16>, u16)> {
    let re_full = regex::Regex::new(r"(?i)(?:s|season)(\d{1,2})(?:e|episode)(\d{1,2})").unwrap();
    let re_episode_only = regex::Regex::new(r"(?i)(?:e|episode)(\d{1,2})").unwrap();

    if let Some(caps) = re_full.captures(name) {
        let season = caps.get(1)?.as_str().parse::<u16>().ok();
        let episode = caps.get(2)?.as_str().parse::<u16>().ok()?;
        Some((season, episode))
    } else if let Some(caps) = re_episode_only.captures(name) {
        let episode = caps.get(1)?.as_str().parse::<u16>().ok()?;
        Some((None, episode))
    } else {
        None
    }
}
fn extract_clean_name(name: &str) -> String {
    use regex::Regex;

    let mut cleaned = name.to_string();

    // Remove file extension early
    if let Some(pos) = cleaned.rfind('.') {
        cleaned.truncate(pos);
    }

    // Remove season + episode tags
    let re_tags =
        Regex::new(r"(?i)(s\d{1,2}e\d{1,2}|season\d{1,2}episode\d{1,2}|e\d{1,2}|episode\d{1,2})")
            .unwrap();
    cleaned = re_tags.replace_all(&cleaned, "").to_string();

    // Remove resolution info
    let re_resolution = Regex::new(r"(?i)(720p|1080p|2160p|480p|4k|8k|hd|uhd)").unwrap();
    cleaned = re_resolution.replace_all(&cleaned, "").to_string();

    // Remove codec info
    let re_codec =
        Regex::new(r"(?i)(x264|x265|h\.?264|h\.?265|hevc|aac|ddp|dts|bluray|webrip|hdr)").unwrap();
    cleaned = re_codec.replace_all(&cleaned, "").to_string();

    // Final cleanup: replace punctuation with spaces, collapse multiple spaces, trim
    cleaned = cleaned
        .replace(['_', '.', '-'], " ")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");

    cleaned.trim().to_string()
}
