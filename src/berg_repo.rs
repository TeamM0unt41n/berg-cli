use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::Context;
use config::Config;

use crate::{berg, models::RepoConfig};

#[derive(Clone)]
pub struct BergRepo {
    client: berg::Client,
    config: RepoConfig,
    path: PathBuf,
}

impl BergRepo {
    pub fn create(server: &str) -> anyhow::Result<Self> {
        unimplemented!();
    }

    pub fn open(path: &PathBuf) -> anyhow::Result<Self> {
        let config = load_config(&path.join(".berg.toml"))?;
        let mut repo = Self {
            client: berg::Client::new(&config.server),
            config,
            path: path.to_owned(),
        };
        repo.try_auth()?;
        Ok(repo)
    }

    fn try_auth(&mut self) -> anyhow::Result<()> {
        let auth_file = self.path.join(".berg.auth");
        if auth_file.exists() {
            let token = fs::read_to_string(&auth_file)?;
            self.client = self.client.authenticate(&token);
        }
        Ok(())
    }

    pub fn authenticate(&mut self, token: &str) -> anyhow::Result<()> {
        let client = self.client.authenticate(token);
        // write authentication token to file
        let mut file = File::create(self.path.join(".berg.auth"))?;
        file.write_all(token.as_bytes())?;
        self.client = client;
        Ok(())
    }

    fn done_path(&self) -> PathBuf {
        self.path.join(".done")
    }
}

fn load_config(path: &PathBuf) -> anyhow::Result<RepoConfig> {
    Config::builder()
        .add_source(config::File::from(path.to_owned()))
        .build()
        .context("could not parse .berg.toml")?
        .try_deserialize::<RepoConfig>()
        .context("could not deserialize repo config")
}
