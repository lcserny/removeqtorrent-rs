use std::sync::Arc;

use async_trait::async_trait;
use eyre::Result;
use mongodb::{Client, bson::{Document, doc, DateTime}};
use tracing::info;

use crate::{torrents::TorrentFile, Settings, downloads::HistoryUpdater};

pub struct MongoUpdater {
    config: Arc<Settings>,
    client: Client
}

impl MongoUpdater {
    pub fn new(config: Arc<Settings>, client: Client) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl HistoryUpdater for MongoUpdater {
    async fn update_history(&self, files: Vec<TorrentFile>) -> Result<()> {
        let database = self.client.database(&self.config.mongodb.database);
        let collection = database.collection::<Document>(&self.config.mongodb.download_collection);

        let docs: Vec<Document> = files.iter()
                    .filter(|t| t.is_media)
                    .map(|t| {
                        doc! { 
                            "file_name": &t.name, 
                            "file_size": t.size as i64,
                            "date_downloaded": DateTime::now(),
                        }
                    }).collect();

        if !docs.is_empty() {
            collection.insert_many(docs, None).await?;
            info!("cache updated for collection {}", &self.config.mongodb.download_collection);
        } else {
            info!("no media files found to insert in cache");
        }

        Ok(())
    }
}