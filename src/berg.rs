use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::PathBuf,
};

use anyhow::Context;
use flate2::bufread::GzDecoder;
use openidconnect::{AuthType, ClientId, IssuerUrl, OAuth2TokenResponse, ProviderMetadata, ResourceOwnerPassword, ResourceOwnerUsername, core::{CoreAuthDisplay, CoreClaimName, CoreClaimType, CoreClientAuthMethod, CoreGrantType, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreJweKeyManagementAlgorithm, CoreProviderMetadata, CoreResponseMode, CoreResponseType, CoreSubjectIdentifierType}};
use serde::de::DeserializeOwned;
use tar::Archive;
use tracing::info;

use crate::models::{Challenge, Instance, SubmitFlagResult};

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

    pub async fn authenticate(&self, uid: &str, token: &str) -> anyhow::Result<Self> {
        let mut clone = self.clone();
        // do the grant
        let issuer = &self.berg_server;
        let http_client = self.http_client.clone();
        let metadata = CoreProviderMetadata::discover_async(IssuerUrl::new(issuer.to_string())?, &http_client).await?;
        let oidc_client = openidconnect::core::CoreClient::from_provider_metadata(metadata, ClientId::new("berg-client".to_string()), None)
            .set_auth_type(AuthType::RequestBody);

        let response = oidc_client.exchange_password(
            &ResourceOwnerUsername::new(uid.to_string()), 
            &ResourceOwnerPassword::new(token.to_string())
        )?
        .request_async(&http_client).await?;
        let password = response.access_token();
        
        clone.token = Some(password.clone().into_secret().to_owned());
        Ok(clone)
    }

    pub fn server_url(&self) -> url::Url {
        url::Url::parse(&self.berg_server).unwrap()
    }

    async fn get(&self, url: &str) -> reqwest::Result<reqwest::Response> {
        let mut request = self.http_client.get(format!("{}{}", self.berg_server, url.trim_prefix("/")));

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);  
        }

        request.send().await
    }

    async fn delete(&self, url: &str) -> reqwest::Result<reqwest::Response> {
        let mut request = self.http_client.delete(format!("{}{}", self.berg_server, url.trim_prefix("/")));

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);  
        }

        request.send().await
    }

    fn post_builder(&self, url: &str) -> reqwest::RequestBuilder {
        let mut request = self
            .http_client
            .post(format!("{}{}", self.berg_server, url.trim_prefix("/")));

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);  
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
    pub async fn get_metadata(&self) -> anyhow::Result<crate::models::Metadata> {
        self.get_json("/api/metadata").await
    }

    pub async fn get_challenges(&self) -> anyhow::Result<Vec<crate::models::Challenge>> {
        self.get_json("/api/challenges").await
    }

    pub async fn get_solves(&self) -> anyhow::Result<Vec<crate::models::PlayerSolve>> {
        self.get_json("/api/solves").await
    }

    pub async fn submit_flag(
        &self,
        challenge: &str,
        flag: &str,
    ) -> anyhow::Result<SubmitFlagResult> {
        self.post_builder("/api/solves")
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

    pub async fn get_self(&self) -> anyhow::Result<crate::models::Player> {
        self.get_json("/api/players/current").await
    }

    pub async fn get_instance(&self) -> anyhow::Result<Instance> {
        self.get_json("/api/instances/current").await
    }

    pub async fn start_instance(&self, challenge: &str) -> anyhow::Result<Instance> {
        self.post_builder("/api/instances/current")
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
        self.delete("/api/instances/current")
            .await
            .context("could not stop instance")?
            .error_for_status()
            .context("server returned an error")
            .map(|_| ())
    }
}
