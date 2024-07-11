use std::{
    fs::{self, create_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use tracing::{info, warn};
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
    repo.sync(true, false).await?;
    Ok(())
}

pub async fn sync(flagdump: bool) -> anyhow::Result<()> {
    let repo = crate::berg_repo::BergRepo::from_env()?;
    repo.sync(false, flagdump).await?;

    Ok(())
}

pub async fn authenticate() -> anyhow::Result<()> {
    let token = inquire::Text::new("Enter your authentication token: ").prompt()?;

    let mut repo = crate::berg_repo::BergRepo::from_env()?;
    repo.authenticate(&token)?;

    Ok(())
}

pub async fn submit(challenge: &str, flag: &str) -> anyhow::Result<()> {
    let repo = crate::berg_repo::BergRepo::from_env()?;
    let result = repo.submit_flag(challenge, flag).await?;

    match result {
        crate::models::SubmitFlagResult::Correct => {
            info!("Flag is correct!");
        }
        crate::models::SubmitFlagResult::Incorrect => {
            info!("Flag is incorrect.");
        }
        _ => {
            warn!("Flag is incorrect or already submitted.");
        }
    }

    Ok(())
}

pub async fn instance_start(challenge: &str) -> anyhow::Result<()> {
    let repo = crate::berg_repo::BergRepo::from_env()?;
    let instance = repo.client.start_instance(challenge).await?;

    info!("Instance started: {:?}", instance);

    Ok(())
}

pub async fn instance_stop() -> anyhow::Result<()> {
    let repo = crate::berg_repo::BergRepo::from_env()?;
    repo.client.stop_instance().await?;

    info!("Instance stopped.");

    Ok(())
}
