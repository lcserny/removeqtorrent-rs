use std::sync::Arc;

use config::Settings;
use eyre::Result;
use futures::try_join;
use mongo::MongoUpdater;
use qtorrent::QTorrentHandler;
use reqwest::Client;
use tracing::info;

use crate::{torrents::TorrentsHandler, downloads::HistoryUpdater};

pub mod qtorrent;
pub mod mongo;
pub mod config;
pub mod torrents;
pub mod downloads;

pub async fn execute(cfg: Arc<Settings>, hash: String) -> Result<()> {
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