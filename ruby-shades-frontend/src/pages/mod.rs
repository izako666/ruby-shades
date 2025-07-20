pub mod home;
pub mod movie;
pub mod show;
pub mod watch;
pub fn tmdb_image_url(path: &str) -> String {
    format!("https://image.tmdb.org/t/p/original{}", path)
}
