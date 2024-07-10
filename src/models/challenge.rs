use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub file_name: String,
    pub download_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSolve {
    pub player_id: String,
    pub solved_at: DateTime<Utc>,
    pub challenge_name: String,
    pub is_first_blood: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamSolve {
    pub player_id: String,
    pub team_id: String,
    pub solved_at: DateTime<Utc>,
    pub challenge_name: String,
}
