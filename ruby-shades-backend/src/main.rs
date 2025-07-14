use crate::config::{Config, load_config, read_config};

mod axum_service;
mod config;
mod directory_parser;
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
