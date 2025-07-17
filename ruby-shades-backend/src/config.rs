use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{fs, sync::Mutex};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub source_dir: String,
    pub address: String,
    pub port: String,
    pub tmdb_auth_token: String,
}

static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));
pub fn load_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string("Config.toml")?;

    let new_config = toml::from_str(&config_str)?;

    let mut config = CONFIG.lock().unwrap();
    *config = new_config;
    Ok(())
}

pub fn read_config() -> Config {
    match &*CONFIG.lock().unwrap() {
        Some(val) => val.clone(),
        None => panic!("CONFIG NOT FOUND!"),
    }
}
