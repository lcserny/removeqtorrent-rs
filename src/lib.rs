use std::{env, fs::OpenOptions, sync::Arc};

use config::{Config, File, Environment};
use eyre::{Context, Result};
use futures::try_join;
use mongo::MongoUpdater;
use qtorrent::QTorrentHandler;
use reqwest::Client;
use serde::Deserialize;
use tracing::info;

use crate::{torrents::TorrentsHandler, downloads::HistoryUpdater};

pub mod qtorrent;
pub mod mongo;

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

pub async fn run(cfg: Arc<Settings>, hash: String) -> Result<()> {
    info!("hash received: {}", &hash);

    let torrent_handler = QTorrentHandler::new(cfg.clone(), Client::new());
    let mongo_client = mongodb::Client::with_uri_str(&cfg.mongodb.connection_url).await?;
    let history_updater = MongoUpdater::new(cfg, mongo_client);

    let sid = torrent_handler.generate_sid().await?;
    let torrents = torrent_handler.list_files(&sid, &hash).await?;

    try_join!(
        history_updater.update_history(torrents), 
        torrent_handler.delete(&sid, &hash, false)
    )?;

    Ok(())
}

pub fn init_logging(log_file_path: &str) -> eyre::Result<()> {
    let file_appender = OpenOptions::new().create(true).write(true).append(true).open(log_file_path)?;
    tracing_subscriber::fmt().with_writer(file_appender).init(); 
    Ok(())
}

pub fn init_config(filename: &str, env_prefix: &str) -> Result<Settings> {
    let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

    return Config::builder()
                .add_source(File::with_name(filename))
                .add_source(File::with_name(&format!("{}_{}", filename, run_mode)).required(false))
                .add_source(Environment::with_prefix(env_prefix))
                .build()?
                .try_deserialize().wrap_err_with(|| format!("failed to create Settings from config proovided: {}", &filename));
}

pub mod torrents {
    use async_trait::async_trait;
    use eyre::Result;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct TorrentFile {
        pub name: String,
        pub size: u64,
        #[serde(skip_deserializing)]
        pub is_media: bool,
    }

    #[async_trait]
    pub trait TorrentsHandler {
        async fn generate_sid(&self) -> Result<String>;
        async fn list_files(&self, sid: &str, hash: &str) -> Result<Vec<TorrentFile>>;
        async fn delete(&self, sid: &str, hash: &str, delete_files: bool) -> Result<()>;
    }
}

pub mod downloads {
    use async_trait::async_trait;
    use eyre::Result;

    use crate::torrents::TorrentFile;

    #[async_trait]
    pub trait HistoryUpdater {
        async fn update_history(&self, files: Vec<TorrentFile>) -> Result<()>;
    }
}