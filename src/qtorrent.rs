use std::path::Path;

use anyhow::{anyhow, Context};
use reqwest::{blocking::Client, header::{SET_COOKIE, COOKIE}};
use tracing::info;

use crate::{Settings, torrents::{TorrentFile, TorrentsHandler}};

pub const SID_KEY: &str = "SID";

pub struct QTorrentHandler<'a> {
    config: &'a Settings,
    http_client: &'a Client,
}

impl <'a> QTorrentHandler<'a> {
    pub fn new(config: &'a Settings, http_client: &'a Client) -> Self {
        Self { 
            config,
            http_client,
        }
    }
}

impl <'a> TorrentsHandler for QTorrentHandler<'a> {
    fn generate_sid(&self) -> Result<String, anyhow::Error> {
        let url = format!("{}/api/v2/auth/login", &self.config.torrent_web_ui.base_url);

        let params = [
            ("username", &self.config.torrent_web_ui.username), 
            ("password", &self.config.torrent_web_ui.password)
        ];

        let resp = self.http_client.post(url).form(&params).send()?;

        let cookies = resp.headers().get(SET_COOKIE)
            .with_context(|| "could not generate SID, no cookies found in response headers")?;

        let cookies_str = cookies.to_str()?;
        if !cookies_str.contains(SID_KEY) {
            return Err(anyhow!("no SID cookie found in response while generating SID"));
        }

        let idx = match cookies_str.find(';') {
            Some(i) => i,
            None => cookies_str.len(),
        };

        let sid = cookies_str[4..idx].to_string();
        info!("SID generated: {}", &sid);

        Ok(sid)
    }

    fn list_files(&self, sid: &str, hash: &str) -> Result<Vec<TorrentFile>, anyhow::Error> {
        let url = format!("{}/api/v2/torrents/files", &self.config.torrent_web_ui.base_url);

        let params = [("hash", hash)];

        let resp = self.http_client.post(url).header(COOKIE, format!("{}={}", SID_KEY, sid).as_str()).form(&params).send()?;
        let resp_body = resp.text()?;
        let mut torrent_files: Vec<TorrentFile> = serde_json::from_str(&resp_body)
            .with_context(|| format!("could not deserialize json: {:?}", &resp_body))?;

        torrent_files.iter_mut()
            .for_each(|tf| {
                let path = Path::new(&self.config.torrent_web_ui.download_root_path)
                    .join(&tf.name);
                tf.is_media = is_video(&path, self.config);
            });

        info!("torrent files retrieved: {:?}", torrent_files);

        Ok(torrent_files)
    }

    fn delete(&self, sid: &str, hash: &str, delete_files: bool) -> Result<(), anyhow::Error> {
        let url = format!("{}/api/v2/torrents/delete", &self.config.torrent_web_ui.base_url);

        let params = [
            ("hashes", hash), 
            ("deleteFiles", &delete_files.to_string())
        ];

        self.http_client.post(url).header(COOKIE, format!("{}={}", SID_KEY, sid).as_str()).form(&params).send()?;

        info!("deleted torrent with hash {}", hash);

        Ok(())
    }
}

fn is_video(path: &Path, config: &Settings) -> bool {
    if let Some(mime) = tree_magic_mini::from_filepath(path) {
        for allowed_mime in &config.video_mime_types {
            if allowed_mime.eq(mime) {
                return true;
            }
        }

        if mime.starts_with("video/") {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{Settings, TorrentWebUI, MongoDb};

    use super::is_video;

    #[test]
    #[ignore = "tmp test mime"]
    fn mime_resolved_ok() {
        let mkv = "/mnt/HDD/Downloads/Loki.S02E02.WEB.x264-TORRENTGALAXY[TGx]/Loki.S02E02.WEB.x264-TORRENTGALAXY.mkv";
        let path = Path::new(mkv);

        let config = Settings { 
            mongodb: MongoDb{ connection_url: String::new(), database: String::new(), download_collection: String::new() }, 
            torrent_web_ui: TorrentWebUI{ base_url: String::new(), username: String::new(), password: String::new(), download_root_path: String::new() }, 
            video_mime_types: vec![] };

        let is_video = is_video(path, &config);
        assert!(is_video);
    }
}