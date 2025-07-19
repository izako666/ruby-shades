use once_cell::unsync::Lazy;
use reqwasm::http::Request;
use serde::Deserialize;
use std::{cell::RefCell, fs};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub backend_url: String,
}

thread_local! {
    static CONFIG: Lazy<RefCell<Option<Config>>> = Lazy::new(|| RefCell::new(None));
}

pub async fn load_config() -> Result<(), anyhow::Error> {
    let response = Request::get("static/Config.toml")
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch config: {e}"))?;

    let config_str = response.text().await?;
    let new_config: Config = toml::from_str(&config_str)?;

    CONFIG.with(|config| {
        *config.borrow_mut() = Some(new_config);
    });

    Ok(())
}

pub fn read_config() -> Config {
    CONFIG.with(|config| config.borrow().clone().expect("Config not loaded yet!"))
}
