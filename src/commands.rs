use std::{
    fs::{self, create_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
/// Initialises a challenge repository in the current directory
pub async fn init(server: &str, path: &Option<String>) -> anyhow::Result<()> {
    let current_dir = std::env::current_dir().context("could not get current directory")?;

    let root_dir = if let Some(path) = path {
        current_dir.join(path)
    } else {
        current_dir
    };

    if root_dir.read_dir()?.next().is_some() {
        // not an empty directory
        bail!("Directory is not empty.");
    }

    // ask for optional authentication token
    let auth_token = inquire::Confirm::new("Do you have an authentication token?").prompt()?;
    let auth_token = if auth_token {
        Some(inquire::Text::new("Enter your authentication token: ").prompt()?)
    } else {
        None
    };

    let repo = crate::berg_repo::BergRepo::create(&root_dir, server, &auth_token)?;
    repo.sync().await?;
    Ok(())
}

pub async fn sync() -> anyhow::Result<()> {
    let repo = crate::berg_repo::BergRepo::from_env()?;
    repo.sync().await?;

    Ok(())
}

pub async fn authenticate() -> anyhow::Result<()> {
    let token = inquire::Text::new("Enter your authentication token: ").prompt()?;

    let mut repo = crate::berg_repo::BergRepo::from_env()?;
    repo.authenticate(&token)?;

    Ok(())
}

fn do_auth(root: &PathBuf) -> Option<String> {
    let auth_file = root.join(".berg.auth");
    if auth_file.exists() {
        let auth = fs::read_to_string(&auth_file).unwrap();
        return Some(auth);
    }
    None
}
