#[cfg(test)]
mod tests {
    use std::{sync::Arc, fs};

    use removeqtorrent::{init_config, qtorrent::{QTorrentHandler, SID_KEY}, torrents::TorrentsHandler};
    use reqwest::{{Client, multipart::{self}}, header::COOKIE};
    use serde::Deserialize;
    use testcontainers::{core::WaitFor, clients, GenericImage};

    const PORT: u16 = 8080;

    #[derive(Deserialize)]
    struct TmpTorrent {
        hash: String,
    }

    fn create_image() -> GenericImage {
        GenericImage::new("linuxserver/qbittorrent", "4.5.2")
                    .with_exposed_port(PORT)
                    .with_volume( "./tests/resources/qBittorrent.conf", "/config/qBittorrent/qBittorrent.conf")
                    .with_wait_for(WaitFor::message_on_stdout("[ls.io-init] done."))
    }

    #[tokio::test]
    async fn can_generate_qtorrent_sid() {
        let docker = clients::Cli::default();
        let container = docker.run(create_image());

        let mut config = init_config("config/settings_test", "RQT_TEST").unwrap();
        config.torrent_web_ui.base_url = format!("http://localhost:{}", container.get_host_port_ipv4(PORT));

        let handler = QTorrentHandler::new(Arc::new(config), Client::new());

        let sid = handler.generate_sid().await;

        assert!(!sid.unwrap().is_empty(), "generated SID is empty");
    }

    #[tokio::test]
    async fn can_delete_qtorrent_by_hash() {
        let docker = clients::Cli::default();
        let container = docker.run(create_image());

        let mut config = init_config("config/settings_test", "RQT_TEST").unwrap();
        config.torrent_web_ui.base_url = format!("http://localhost:{}", container.get_host_port_ipv4(PORT));
        let config = Arc::new(config);

        let http_client = Client::new();
        let handler = QTorrentHandler::new(config.clone(), Client::new());

        let sid = handler.generate_sid().await.unwrap();
        let sid_cookie = format!("{}={}", SID_KEY, sid);

        let file_part = multipart::Part::bytes(fs::read("./tests/resources/ubuntu-server.iso.torrent").unwrap()).file_name("ubuntu-server.iso.torrent");
        let form = multipart::Form::new().part("torrents", file_part);
        let add_url = format!("{}/api/v2/torrents/add", config.torrent_web_ui.base_url);
        http_client.post(add_url).header(COOKIE, &sid_cookie)
            .multipart(form).send().await.unwrap();

        let info_url = format!("{}/api/v2/torrents/info", config.torrent_web_ui.base_url);
        let resp = http_client.post(&info_url).header(COOKIE, &sid_cookie)
            .send().await.unwrap();
        let resp_torrents: Vec<TmpTorrent> = resp.json().await.unwrap();

        assert_eq!(1, resp_torrents.len());
        assert!(!&resp_torrents[0].hash.is_empty());

        handler.delete(&sid, &resp_torrents[0].hash, true).await.unwrap();

        let resp = http_client.post(&info_url).header(COOKIE, &sid_cookie)
            .send().await.unwrap();
        let resp_torrents: Vec<TmpTorrent> = resp.json().await.unwrap();

        assert_eq!(0, resp_torrents.len());
    }

    #[tokio::test]
    #[ignore = "to list files we need to wait for actual downloading to happen"]
    async fn can_list_files_in_torrent() {
        let docker = clients::Cli::default();
        let container = docker.run(create_image());

        let mut config = init_config("config/settings_test", "RQT_TEST").unwrap();
        config.torrent_web_ui.base_url = format!("http://localhost:{}", container.get_host_port_ipv4(PORT));
        let config = Arc::new(config);

        let http_client = Client::new();
        let handler = QTorrentHandler::new(config.clone(), Client::new());

        let sid = handler.generate_sid().await.unwrap();
        let sid_cookie = format!("{}={}", SID_KEY, sid);

        let file_part = multipart::Part::bytes(fs::read("./tests/resources/ubuntu-server.iso.torrent").unwrap()).file_name("ubuntu-server.iso.torrent");
        let form = multipart::Form::new().part("torrents", file_part);
        let add_url = format!("{}/api/v2/torrents/add", config.torrent_web_ui.base_url);
        http_client.post(add_url).header(COOKIE, &sid_cookie)
            .multipart(form).send().await.unwrap();

        let info_url = format!("{}/api/v2/torrents/info", config.torrent_web_ui.base_url);
        let resp = http_client.post(&info_url).header(COOKIE, &sid_cookie)
            .send().await.unwrap();
        let resp_torrents: Vec<TmpTorrent> = resp.json().await.unwrap();

        let torrent_files = handler.list_files(&sid, &resp_torrents[0].hash).await.unwrap();

        assert_eq!(1, torrent_files.len());
        assert!(!torrent_files[0].is_media);
        assert!(!torrent_files[0].name.is_empty());
        assert!(torrent_files[0].size > 0);
    }
}