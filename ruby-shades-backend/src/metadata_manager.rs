use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

use crate::config::read_config;

#[derive(Debug, Deserialize)]
struct SearchResponses {
    results: Vec<SearchResult>,
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    id: u32,
    backdrop_path: Option<String>,
    poster_path: Option<String>,
    title: String,
}
#[derive(Debug, Deserialize)]
pub struct TvDetails {
    pub backdrop_path: Option<String>,
    pub name: String,
    pub number_of_seasons: u32,
    pub overview: String,
}

pub async fn search_tmdb_movie(
    query: &str,
) -> Result<Option<SearchResult>, Box<dyn std::error::Error>> {
    let token = read_config().imdb_auth_token;

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
) -> Result<Option<SearchResult>, Box<dyn std::error::Error>> {
    let token = read_config().imdb_auth_token;

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

    let search_results: SearchResponses = response.json().await?;

    Ok(search_results.results.into_iter().next())
}

pub async fn fetch_tv_series_details(
    series_id: u32,
) -> Result<TvDetails, Box<dyn std::error::Error>> {
    let token = read_config().imdb_auth_token;

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
