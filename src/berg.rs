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

use crate::models::{Challenge, Instance, SubmitFlagResult};

#[derive(Clone)]
pub struct Client {
    http_client: reqwest::Client,
    berg_server: String,
    token: Option<String>,
    _basic_auth: Option<(String, String)>,
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
            _basic_auth: None,
        }
    }

    pub fn authenticate(&self, token: &str) -> Self {
        let mut clone = self.clone();
        clone.token = Some(token.to_owned());
        clone
    }

    pub fn basic_auth(&self, username: &str, password: &str) -> Self {
        let mut clone = self.clone();
        clone._basic_auth = Some((username.to_owned(), password.to_owned()));
        clone
    }

    pub fn server_url(&self) -> url::Url {
        url::Url::parse(&self.berg_server).unwrap()
    }

    async fn get(&self, url: &str) -> reqwest::Result<reqwest::Response> {
        let mut request = self.http_client.get(format!("{}{}", self.berg_server, url));

        if let Some(token) = &self.token {
            request = request.header("Cookie", format!("berg-auth={}", token));
        }

        if let Some((username, password)) = &self._basic_auth {
            request = request.basic_auth(username, Some(password));
        }

        request.send().await
    }

    fn post_builder(&self, url: &str) -> reqwest::RequestBuilder {
        let mut request = self
            .http_client
            .post(format!("{}{}", self.berg_server, url));

        if let Some(token) = &self.token {
            request = request.header("Cookie", format!("berg-auth={}", token));
        }

        if let Some((username, password)) = &self._basic_auth {
            request = request.basic_auth(username, Some(password));
        }

        request
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

    pub async fn submit_flag(
        &self,
        challenge: &str,
        flag: &str,
    ) -> anyhow::Result<SubmitFlagResult> {
        self.post_builder("/api/v1/flag")
            .json(&serde_json::json!({
                "challenge": challenge,
                "flag": flag,
            }))
            .send()
            .await
            .context("could not submit flag")?
            .error_for_status()
            .context("server returned an error")?
            .json()
            .await
            .context("could not deserialize json")
    }

    pub async fn get_self(&self) -> anyhow::Result<crate::models::PlayerSummary> {
        self.get_json("/api/v1/self").await
    }

    pub async fn start_instance(&self, challenge: &str) -> anyhow::Result<Instance> {
        self.post_builder("/api/v1/challengeInstance/start")
            .json(&serde_json::json!({
                "challenge": challenge,
            }))
            .send()
            .await
            .context("could not start instance")?
            .json()
            .await
            .context("could not deserialise json")
    }

    pub async fn stop_instance(&self) -> anyhow::Result<()> {
        self.post_builder("/api/v1/challengeInstance/stop")
            .send()
            .await
            .context("could not stop instance")?
            .error_for_status()
            .context("server returned an error")
            .map(|_| ())
    }
}
