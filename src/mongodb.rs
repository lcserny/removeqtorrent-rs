use mongodb::{sync::Client, bson::{Document, doc, DateTime}};
use tracing::info;

use crate::{torrents::TorrentFile, Settings, downloads::HistoryUpdater};

pub struct MongoUpdater<'a> {
    config: &'a Settings,
    client: Client
}

impl <'a> MongoUpdater<'a> {
    pub fn new(config: &'a Settings) -> Result<Self, anyhow::Error> {
        Ok(Self { 
            config,
            client: Client::with_uri_str(&config.mongodb.connection_url)?
        })
    }
}

impl <'a> HistoryUpdater for MongoUpdater<'a> {
    fn update_history(&self, files: Vec<TorrentFile>) -> Result<(), anyhow::Error> {
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
            collection.insert_many(docs, None)?;
            info!("cache updated for collection {}", &self.config.mongodb.download_collection);
        } else {
            info!("no media files found to insert in cache");
        }

        Ok(())
    }
}