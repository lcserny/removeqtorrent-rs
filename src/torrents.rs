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
