mod berg;
mod repo_config;

pub use berg::*;
pub use repo_config::RepoConfig;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSummary {
    pub player: Player,
    pub challenge_instance: Option<Instance>,
}
