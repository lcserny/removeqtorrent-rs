#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use futures::TryStreamExt;
    use mongodb::{Client, bson::{Document, doc}};
    use removeqtorrent::{mongo::MongoUpdater, downloads::HistoryUpdater, torrents::TorrentFile, config::init_config};
    use testcontainers::{GenericImage, core::WaitFor, clients};

    const PORT: u16 = 27017;
    const USER: &str = "root";
    const PASS: &str = "rootpass";

    fn create_image() -> GenericImage {
        GenericImage::new("mongo", "5.0")
                    .with_exposed_port(PORT)
                    .with_env_var("MONGO_INITDB_ROOT_USERNAME", USER)
                    .with_env_var("MONGO_INITDB_ROOT_PASSWORD", PASS)
                    .with_wait_for(WaitFor::message_on_stdout("Waiting for connections"))
    }

    #[tokio::test]
    async fn can_update_history() {
        let docker = clients::Cli::default();
        let container = docker.run(create_image());

        let mut config = init_config("config/settings_test", "RQT_TEST").unwrap();
        config.mongodb.connection_url = format!("mongodb://{}:{}@localhost:{}/?retryWrites=true&w=majority", 
            USER, PASS, container.get_host_port_ipv4(PORT));
        let config = Arc::new(config);

        let client = Client::with_uri_str(&config.mongodb.connection_url).await.unwrap();
        let updater = MongoUpdater::new(config.clone(), client.clone());

        let name = "name1";
        let size = 1;
        let is_media = true;

        updater.update_history(vec![TorrentFile {name: name.to_string(), size, is_media}]).await.unwrap();

        let client = Client::with_uri_str(&config.mongodb.connection_url).await.unwrap();
        let database = client.database(&config.mongodb.database);
        let collection = database.collection::<Document>(&config.mongodb.download_collection);

        assert_eq!(1, collection.count_documents(None, None).await.unwrap());
        
        let mut cursor = collection.find(doc!("file_name":name,"file_size":size as i64),None).await.unwrap();
        let mut results = vec![];
        while let Some(doc) = cursor.try_next().await.unwrap() {
            results.push(doc);
        }
        assert_eq!(1, results.len());

        let d = &results[0];
        assert_eq!(name, d.get("file_name").unwrap().as_str().unwrap());
        assert_eq!(size as i64, d.get("file_size").unwrap().as_i64().unwrap());
        assert!(d.get("date_downloaded").unwrap().as_datetime().is_some());
    }
}