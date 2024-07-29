use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepoConfig {
    pub server: String,
    pub basic_auth: Option<(String, String)>,
}
