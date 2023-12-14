use async_trait::async_trait;
use eyre::Result;

use crate::torrents::TorrentFile;

#[async_trait]
pub trait HistoryUpdater {
    async fn update_history(&self, files: Vec<TorrentFile>) -> Result<()>;
}
