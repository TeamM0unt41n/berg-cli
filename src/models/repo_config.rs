use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RepoConfig {
    pub server: String,
}
