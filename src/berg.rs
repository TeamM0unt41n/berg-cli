use anyhow::Context;
use serde::de::DeserializeOwned;

use crate::models::{Challenge, Instance, Metadata, Solve, AddSolveRequest, InstanceStartRequest, CurrentPlayer, CurrentTeam, SubmitFlagResult};

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
    pub async fn get_challenges(&self) -> anyhow::Result<Vec<Challenge>> {
        self.get_json("/api/challenges").await
    }

    pub async fn get_challenge(&self, name: &str) -> anyhow::Result<Challenge> {
        self.get_json(&format!("/api/challenges/{}", name)).await
    }

    pub async fn get_metadata(&self) -> anyhow::Result<Metadata> {
        self.get_json("/api/metadata").await
    }

    pub async fn get_current_player(&self) -> anyhow::Result<CurrentPlayer> {
        self.get_json("/api/players/current").await
    }

    pub async fn get_current_team(&self) -> anyhow::Result<CurrentTeam> {
        self.get_json("/api/teams/current").await
    }

    pub async fn submit_flag(&self, challenge: &str, flag: &str) -> anyhow::Result<Solve> {
        let request = AddSolveRequest {
            challenge: Some(challenge.to_string()),
            flag: Some(flag.to_string()),
        };
        
        self.post_builder("/api/solves")
            .json(&request)
            .send()
            .await
            .context("could not submit flag")?
            .error_for_status()
            .context("server returned an error")?
            .json()
            .await
            .context("could not deserialize json")
    }

    pub async fn start_instance(&self, challenge: &str) -> anyhow::Result<Instance> {
        let request = InstanceStartRequest {
            challenge: Some(challenge.to_string()),
        };
        
        self.post_builder("/api/instances/current")
            .json(&request)
            .send()
            .await
            .context("could not start instance")?
            .json()
            .await
            .context("could not deserialise json")
    }

    pub async fn stop_instance(&self) -> anyhow::Result<Instance> {
        self.http_client
            .delete(format!("{}/api/instances/current", self.berg_server))
            .send()
            .await
            .context("could not stop instance")?
            .error_for_status()
            .context("server returned an error")?
            .json()
            .await
            .context("could not deserialize json")
    }

    pub async fn get_current_instance(&self) -> anyhow::Result<Instance> {
        self.get_json("/api/instances/current").await
    }

    // Legacy method for backward compatibility - converts new API to old CTF format
    pub async fn get_ctf(&self) -> anyhow::Result<crate::models::Ctf> {
        let challenges = self.get_challenges().await?;
        let metadata = self.get_metadata().await?;
        
        // Group challenges by category
        let mut challenges_by_category = std::collections::HashMap::new();
        for challenge in challenges {
            if let Some(categories) = &challenge.categories {
                for category in categories {
                    challenges_by_category
                        .entry(category.clone())
                        .or_insert_with(Vec::new)
                        .push(challenge.clone());
                }
            } else {
                challenges_by_category
                    .entry("Uncategorized".to_string())
                    .or_insert_with(Vec::new)
                    .push(challenge);
            }
        }
        
        Ok(crate::models::Ctf {
            start: metadata.start,
            end: metadata.end,
            server_time: chrono::Utc::now(), // This is not available in new API
            freeze_start: metadata.freeze_start,
            freeze_end: metadata.freeze_end,
            teams: metadata.teams,
            challenges_by_category,
        })
    }

    // Legacy method for backward compatibility - returns SubmitFlagResult
    pub async fn submit_flag_legacy(&self, challenge: &str, flag: &str) -> anyhow::Result<SubmitFlagResult> {
        match self.submit_flag(challenge, flag).await {
            Ok(_) => Ok(SubmitFlagResult::Correct),
            Err(e) => {
                // Try to parse the error to determine the specific failure reason
                // For now, we'll default to Incorrect
                let error_msg = e.to_string();
                if error_msg.contains("400") {
                    Ok(SubmitFlagResult::Incorrect)
                } else if error_msg.contains("429") {
                    Ok(SubmitFlagResult::RateLimited)
                } else if error_msg.contains("409") {
                    Ok(SubmitFlagResult::AlreadySolved)
                } else {
                    Ok(SubmitFlagResult::Incorrect)
                }
            }
        }
    }
}
