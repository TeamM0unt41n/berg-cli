use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub version: String,
    pub event_name: String,
    pub event_organiser: String,
    pub event_logo_url: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub allow_anonymous_access: bool,
    pub freeze_start: Option<DateTime<Utc>>,
    pub freeze_end: Option<DateTime<Utc>>,
    pub player_attributes: Vec<String>,
    pub challenge_maximum_value: i32,
    pub challenge_minimum_value: i32,
    pub challenge_solves_before_minimum: i32,
    pub teams: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    pub name: String,
    pub display_name: String,
    pub author: String,
    pub description: String,
    pub hide_until: Option<DateTime<Utc>>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub event: String,
    pub difficulty: String,
    pub flag_format: String,
    pub attachments: Vec<Attachment>,
    pub has_remote: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub file_name: String,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSolve {
    pub player_id: String,
    pub solved_at: DateTime<Utc>,
    pub challenge_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamSolve {
    pub player_id: String,
    pub team_id: String,
    pub solved_at: DateTime<Utc>,
    pub challenge_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub id: String,
    pub name: String,
    pub roles: Option<Vec<String>>,
    pub federated_id: Option<String>,
    pub api_key_placeholder: Option<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerAttribute {
    pub name: String,
    pub public: bool,
    pub values: Vec<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: Option<String>,
    pub player_id: String,
    pub name: String,
    pub status: i32,
    pub services: Vec<Service>,
    pub timeout: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub terminated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub name: Option<String>,
    pub port: i32,
    pub protocol: String,
    pub hostname: String,
    pub app_protocol: String,
    pub tls: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum SubmitFlagResult {
    Correct,
    Incorrect,
    RateLimited,
    AlreadySolved,
    CtfNotStarted,
    CtfHasEnded,
}

impl<'de> Deserialize<'de> for SubmitFlagResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: i32 = Deserialize::deserialize(deserializer)?;
        match value {
            0 => Ok(SubmitFlagResult::Correct),
            1 => Ok(SubmitFlagResult::Incorrect),
            2 => Ok(SubmitFlagResult::RateLimited),
            3 => Ok(SubmitFlagResult::AlreadySolved),
            4 => Ok(SubmitFlagResult::CtfNotStarted),
            5 => Ok(SubmitFlagResult::CtfHasEnded),
            _ => Err(serde::de::Error::custom("invalid SubmitFlagResult")),
        }
    }
}
