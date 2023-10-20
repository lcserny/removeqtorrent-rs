use std::{env, thread::{self}};

use anyhow::Context;
use config::{Config, File, Environment};
use downloads::HistoryUpdater;
use mongodb::MongoUpdater;
use qtorrent::QTorrentHandler;
use reqwest::blocking::Client;
use serde::Deserialize;
use tracing::{error, info};

use crate::torrents::TorrentsHandler;

pub mod qtorrent;
pub mod mongodb;

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

pub fn run(cfg: Settings, hash: String) -> Result<(), anyhow::Error> {
    info!("hash received: {}", &hash);

    let http_client = Client::new();
    let torrent_handler = QTorrentHandler::new(&cfg, &http_client);
    let history_updater = MongoUpdater::new(&cfg)?;

    let sid = torrent_handler.generate_sid()?;
    let torrents = torrent_handler.list_files(&sid, &hash)?;

    let (update_result, delete_result) = thread::scope(|scp| {
        let update_handle = scp.spawn(|| {
            history_updater.update_history(torrents)
        });
        let delete_handle = scp.spawn(|| {
            torrent_handler.delete(&sid, &hash, false)
        });
        return (update_handle.join().unwrap(), delete_handle.join().unwrap());
    });

    update_result?;
    delete_result?;

    Ok(())
}

pub fn init_logging(dir: &str, prefix: &str) {
    let file_appender = tracing_appender::rolling::daily(dir, prefix);
    // let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    // tracing_subscriber::fmt().with_writer(non_blocking).init();
    tracing_subscriber::fmt().with_writer(file_appender).init();
}

pub fn init_config(filename: &str, env_prefix: &str) -> Result<Settings, anyhow::Error> {
    let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

    return Config::builder()
                .add_source(File::with_name(filename))
                .add_source(File::with_name(&format!("{}_{}", filename, run_mode)).required(false))
                .add_source(Environment::with_prefix(env_prefix))
                .build()?
                .try_deserialize().with_context(|| "failed to create Settings from config proovided");
}

pub fn log_and_fail(e: anyhow::Error, error_code: i32) {
    error!("{}", e.to_string());
    std::process::exit(error_code);
}

pub mod torrents {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct TorrentFile {
        pub name: String,
        pub size: u64,
        #[serde(skip_deserializing)]
        pub is_media: bool,
    }

    pub trait TorrentsHandler {
        fn generate_sid(&self) -> Result<String, anyhow::Error>;
        fn list_files(&self, sid: &str, hash: &str) -> Result<Vec<TorrentFile>, anyhow::Error>;
        fn delete(&self, sid: &str, hash: &str, delete_files: bool) -> Result<(), anyhow::Error>;
    }
}

pub mod downloads {
    use crate::torrents::TorrentFile;

    pub trait HistoryUpdater {
        fn update_history(&self, files: Vec<TorrentFile>) -> Result<(), anyhow::Error>;
    }
}