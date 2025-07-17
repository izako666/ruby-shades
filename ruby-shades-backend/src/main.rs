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
    let path_obj = directory_parser::PATH_OBJECT.lock().unwrap();
    if let Some(path) = path_obj.clone() {
        let _ = database::DB.clear();

        let _ = metadata_manager::transcode_path_object(&path, None).await;
    }

    axum_service::initialize().await;
}
