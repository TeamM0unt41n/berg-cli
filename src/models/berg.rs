use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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
    pub player_attributes: Option<Vec<PlayerAttribute>>,
    pub freeze_start: Option<DateTime<Utc>>,
    pub freeze_end: Option<DateTime<Utc>>,
    pub teams: bool,
    pub challenge_maximum_value: i32,
    pub challenge_minimum_value: i32,
    pub challenge_solves_before_minimum: i32,
}

// Legacy struct for backward compatibility
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
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub hide_until: Option<DateTime<Utc>>,
    pub categories: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub event: Option<String>,
    pub difficulty: Option<String>,
    pub flag_format: Option<String>,
    pub attachments: Option<Vec<Attachment>>,
    pub has_remote: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub file_name: Option<String>,
    pub download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Solve {
    pub id: Uuid,
    pub player_id: Uuid,
    pub solved_at: DateTime<Utc>,
    pub challenge_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    pub id: Uuid,
    pub player_id: Uuid,
    pub submitted_at: DateTime<Utc>,
    pub challenge_name: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub title: Option<String>,
    pub path: Option<String>,
    pub index: i32,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSolveRequest {
    pub challenge: Option<String>,
    pub flag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStartRequest {
    pub challenge: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamCreateRequest {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinTeamRequest {
    pub join_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributesUpdateRequest {
    pub attributes: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub id: Uuid,
    pub name: Option<String>,
    pub attributes: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentPlayer {
    pub id: Uuid,
    pub name: Option<String>,
    pub roles: Option<Vec<String>>,
    pub federated_id: Option<String>,
    pub api_key_placeholder: Option<String>,
    pub attributes: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: Uuid,
    pub name: Option<String>,
    pub players: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentTeam {
    pub id: Uuid,
    pub name: Option<String>,
    pub join_token: Option<String>,
    pub players: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerAttribute {
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub public: bool,
    pub required: bool,
    pub values: Option<Vec<PlayerAttributeValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerAttributeValue {
    pub value: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: Option<Uuid>,
    pub player_id: Option<Uuid>,
    pub name: Option<String>,
    pub status: InstanceState,
    pub services: Option<Vec<Service>>,
    pub timeout: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub terminated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstanceState {
    #[serde(rename = "0")]
    Pending = 0,
    #[serde(rename = "1")]
    Running = 1,
    #[serde(rename = "2")]
    Stopped = 2,
    #[serde(rename = "3")]
    Failed = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub name: Option<String>,
    pub hostname: Option<String>,
    pub port: i32,
    pub protocol: Option<String>,
    pub app_protocol: Option<String>,
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
