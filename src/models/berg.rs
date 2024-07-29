use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ctf {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub server_time: DateTime<Utc>,
    pub freeze_start: Option<DateTime<Utc>>,
    pub freeze_end: Option<DateTime<Utc>>,
    pub teams: bool,
    pub challenges_by_category: HashMap<String, Vec<Challenge>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    pub name: String,
    pub author: String,
    pub description: String,
    pub categories: Vec<String>,
    pub difficulty: String,
    pub flag_format: String,
    pub attachments: Vec<Attachment>,
    pub value: i32,
    pub solved_by_team: bool,
    pub solved_by_player: bool,
    pub instantiatable: bool,
    pub player_solves: Vec<PlayerSolve>,
    pub team_solves: Vec<TeamSolve>,
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
    pub is_first_blood: bool,
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
    pub team_id: Option<String>,
    pub discord_id: Option<String>,
    pub attributes: HashMap<String, String>,
    pub required_attributes: Vec<PlayerAttribute>,
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
    pub name: Option<String>,
    pub status: i32,
    pub services: Vec<Service>,
    pub instance_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub name: Option<String>,
    pub port: i32,
    pub protocol: String,
    pub hostname: String,
    pub app_protocol: String,
    pub vhost: bool,
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
