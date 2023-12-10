use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use eyre::{ContextCompat, WrapErr, eyre, Result};
use reqwest::{Client, header::{SET_COOKIE, COOKIE}};
use tracing::{info, warn};

use crate::{Settings, torrents::{TorrentFile, TorrentsHandler}};

pub const SID_KEY: &str = "SID";

pub struct QTorrentHandler {
    config: Arc<Settings>,
    http_client: Client,
}

impl QTorrentHandler {
    pub fn new(config: Arc<Settings>, http_client: Client) -> Self {
        Self { config, http_client, }
    }
}

#[async_trait]
impl TorrentsHandler for QTorrentHandler {
    async fn generate_sid(&self) -> Result<String> {
        let url = format!("{}/api/v2/auth/login", &self.config.torrent_web_ui.base_url);

        let params = [
            ("username", &self.config.torrent_web_ui.username), 
            ("password", &self.config.torrent_web_ui.password)
        ];

        let resp = self.http_client.post(url).form(&params).send().await?;

        let cookies = resp.headers().get(SET_COOKIE)
            .wrap_err_with(|| "could not generate SID, no cookies found in response headers")?;

        let cookies_str = cookies.to_str()?;
        if !cookies_str.contains(SID_KEY) {
            return Err(eyre!("no SID cookie found in response while generating SID"));
        }

        let idx = match cookies_str.find(';') {
            Some(i) => i,
            None => cookies_str.len(),
        };

        let sid = cookies_str[4..idx].to_string();
        info!("SID generated: {}", &sid);

        Ok(sid)
    }

    async fn list_files(&self, sid: &str, hash: &str) -> Result<Vec<TorrentFile>> {
        let url = format!("{}/api/v2/torrents/files", &self.config.torrent_web_ui.base_url);

        let params = [("hash", hash)];

        let resp = self.http_client.post(url).header(COOKIE, format!("{}={}", SID_KEY, sid).as_str()).form(&params).send().await?;
        let resp_body = resp.text().await?;
        let mut torrent_files: Vec<TorrentFile> = serde_json::from_str(&resp_body)
            .wrap_err_with(|| format!("could not deserialize json: {:?}", &resp_body))?;

        torrent_files.iter_mut()
            .for_each(|tf| {
                let path = Path::new(&self.config.torrent_web_ui.download_root_path)
                    .join(&tf.name);
                tf.is_media = is_video(&path, &self.config);
            });

        info!("torrent files retrieved: {:?}", torrent_files);

        Ok(torrent_files)
    }

    async fn delete(&self, sid: &str, hash: &str, delete_files: bool) -> Result<()> {
        let url = format!("{}/api/v2/torrents/delete", &self.config.torrent_web_ui.base_url);

        let params = [
            ("hashes", hash), 
            ("deleteFiles", &delete_files.to_string())
        ];

        self.http_client.post(url).header(COOKIE, format!("{}={}", SID_KEY, sid).as_str()).form(&params).send().await?;

        info!("deleted torrent with hash {}", hash);

        Ok(())
    }
}

fn is_video(path: &Path, config: &Settings) -> bool {
    let ftype = match infer::get_from_path(path) {
        Ok(ftype) => ftype,
        Err(e) => {
            warn!("error occurred when infering file type: {:?}", e);
            return false
        },
    };

    if let Some(mime) = ftype {
        for allowed_mime in &config.video_mime_types {
            if allowed_mime == mime.mime_type() {
                return true;
            }
        }
        if mime.mime_type().starts_with("video/") {
            return true;
        }
    }

    false
}