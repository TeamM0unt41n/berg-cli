use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::PathBuf,
};

use anyhow::Context;
use flate2::bufread::GzDecoder;
use serde::de::DeserializeOwned;
use tar::Archive;
use tracing::info;

use crate::models::Challenge;

#[derive(Clone)]
pub struct Client {
    http_client: reqwest::Client,
    berg_server: String,
    token: Option<String>,
}

impl Client {
    pub fn new<T: AsRef<str>>(berg_server: T) -> Self {
        static APP_USER_AGENT: &str =
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        let http_client = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()
            .unwrap();
        Client {
            http_client,
            berg_server: berg_server.as_ref().to_owned(),
            token: None,
        }
    }

    pub fn authenticate(&self, token: &str) -> Self {
        let mut clone = self.clone();
        clone.token = Some(token.to_owned());
        clone
    }

    fn server_url(&self) -> url::Url {
        url::Url::parse(&self.berg_server).unwrap()
    }

    async fn get(&self, url: &str) -> reqwest::Result<reqwest::Response> {
        let mut request = self.http_client.get(format!("{}{}", self.berg_server, url));

        if let Some(token) = &self.token {
            request = request.header("Cookie", format!("berg-auth={}", token));
        }

        request.send().await
    }

    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> anyhow::Result<T> {
        self.get(url)
            .await
            .context("could not perform request")?
            .json()
            .await
            .context("could not deserialise json")
    }
}

impl Client {
    pub async fn get_ctf(&self) -> anyhow::Result<crate::models::Ctf> {
        self.get_json("/api/v1/ctf").await
    }
}

impl Client {
    pub async fn create_challenge(
        &self,
        challenge: &Challenge,
        challenge_dir: &PathBuf,
    ) -> anyhow::Result<()> {
        create_dir_all(challenge_dir)?;

        // readme file
        let readme_file = challenge_dir.join("README.md");
        let mut readme_file = File::create(&readme_file)?;
        let description = html2md::parse_html(&challenge.description);
        let readme_content = format!(
            r"# {}

By **{}**

## Description

{}

",
            &challenge.name, &challenge.author, &description
        );
        readme_file.write_all(readme_content.as_bytes())?;

        info!("created challenge {}", &challenge.name);
        for attachment in &challenge.attachments {
            let url = self.server_url().join(&attachment.download_url)?;
            let file: bytes::Bytes = reqwest::get(url).await?.bytes().await?;
            info!("grabbed attachment {}", &attachment.file_name);
            if attachment.file_name.ends_with(".tar.gz") {
                if untar_file(file, challenge, challenge_dir).is_err() {
                    info!(
                        "could not extract supposed archive in challenge {}: {}",
                        &challenge.name, attachment.file_name
                    );
                }
            } else {
                // download file normally
                info!(
                    "non conformant file found in challenge {}: {}",
                    &challenge.name, attachment.file_name
                );
            }
        }
        Ok(())
    }
}

fn untar_file(
    file: bytes::Bytes,
    challenge: &Challenge,
    challenge_dir: &PathBuf,
) -> anyhow::Result<()> {
    let tar = GzDecoder::new(&file[..]);
    let mut archive = Archive::new(tar);
    if archive
        .entries()?
        .flatten()
        .flat_map(|e| e.path().map(|e| e.into_owned()))
        .all(|e| e.starts_with(&challenge.name))
    {
        // extract into parent dir
        let tar = GzDecoder::new(&file[..]);
        let mut archive = Archive::new(tar);
        archive.unpack(challenge_dir.parent().unwrap())?;
    } else {
        // extract into dir
        let tar = GzDecoder::new(&file[..]);
        let mut archive = Archive::new(tar);
        archive.unpack(challenge_dir)?;
    }
    Ok(())
}
