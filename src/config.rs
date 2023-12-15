use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MongoDb {
    pub connection_url: String,
    pub database: String,
    pub download_collection: String,
}

#[derive(Debug, Deserialize)]
pub struct TorrentWebUI {
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub download_root_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub mongodb: MongoDb,
    pub torrent_web_ui: TorrentWebUI,
    pub video_mime_types: Vec<String>,
}