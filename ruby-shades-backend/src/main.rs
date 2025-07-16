use crate::config::{Config, load_config, read_config};

mod axum_service;
mod config;
mod database;
mod directory_parser;
mod metadata_manager;
#[tokio::main]
async fn main() {
    match load_config() {
        Ok(_) => (),
        Err(e) => panic!("LOADING FAILED!: {}", e),
    }
    let config: Config = read_config();
    println!("source_dir is {}", config.source_dir);

    directory_parser::initialize(&config.source_dir);

    axum_service::initialize().await;
}
