use anyhow::{bail, Context};
use fancy::printcoln;
use tracing::{info, warn};
/// Initialises a challenge repository in the current directory
pub async fn init(server: &str, path: &Option<String>) -> anyhow::Result<()> {
    let current_dir = std::env::current_dir().context("could not get current directory")?;

    let root_dir = if let Some(path) = path {
        current_dir.join(path)
    } else {
        current_dir
    };

    if root_dir.exists() && root_dir.read_dir()?.next().is_some() {
        // not an empty directory
        bail!("Directory is not empty.");
    }

    // ask for optional authentication token
    let auth_token = inquire::Confirm::new("Do you have an authentication token?").prompt()?;
    let auth_token = if auth_token {
        let uid = inquire::Text::new("Enter your uid: ").prompt()?;
        let token = inquire::Password::new("Enter your authentication token: ").prompt()?;
        Some((uid, token))
    } else {
        None
    };

    let repo = crate::berg_repo::BergRepo::create(&root_dir, server, &auth_token).await?;
    println!("Repository initialised at {}", root_dir.display());
    repo.sync(true, false).await?;
    println!("Repository synchronised.");
    Ok(())
}

pub async fn sync(flagdump: bool) -> anyhow::Result<()> {
    let repo = crate::berg_repo::BergRepo::from_env().await?;
    repo.sync(false, flagdump).await?;

    Ok(())
}

pub async fn authenticate() -> anyhow::Result<()> {
    let uid = inquire::Text::new("Enter your uid: ").prompt()?;
    let token = inquire::Text::new("Enter your authentication token: ").prompt()?;

    let mut repo = crate::berg_repo::BergRepo::from_env().await?;
    repo.authenticate(&uid, &token).await?;

    Ok(())
}

pub async fn submit(challenge: &str, flag: &str) -> anyhow::Result<()> {
    let repo = crate::berg_repo::BergRepo::from_env().await?;
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

pub async fn instance_start(challenge: &str, force: bool) -> anyhow::Result<()> {
    let repo = crate::berg_repo::BergRepo::from_env().await?;

    let status = repo.client.get_instance().await?;
    if !status.name.is_empty() {
        if status.name == challenge {
            printcoln!("Already running the same challenge instance.");
            return Ok(());
        } else if force {
            printcoln!("Stopping current instance.");
            repo.client.stop_instance().await?;
        } else {
            bail!("Currently running a different challenge instance.");
        }
    }

    let instance = repo.client.start_instance(challenge).await?;

    printcoln!("Instance started: {:?}", instance);

    Ok(())
}

pub async fn instance_stop() -> anyhow::Result<()> {
    let repo = crate::berg_repo::BergRepo::from_env().await?;
    repo.client.stop_instance().await?;

    info!("Instance stopped.");

    Ok(())
}

pub async fn instance_info() -> anyhow::Result<()> {
    let repo = crate::berg_repo::BergRepo::from_env().await?;
    let status = repo.client.get_instance().await?;
    if !status.name.is_empty() {
        printcoln!("Challenge instance: {}", status.name);
        for service in status.services {
            if let Some(name) = &service.name {
                printcoln!(
                    "{}: {}://{}:{}",
                    &name,
                    &service.protocol,
                    &service.hostname,
                    &service.port
                );
            } else {
                printcoln!(
                    "{}://{}:{}",
                    &service.protocol,
                    &service.hostname,
                    &service.port
                );
            }
        }
    } else {
        printcoln!("No challenge instance running.");
    }
    Ok(())
}

pub async fn instance_exploit(
    _script: &str,
    _cmd: &str,
    _start: bool,
    _stop: bool,
    force: bool,
) -> anyhow::Result<()> {
    let repo = crate::berg_repo::BergRepo::from_env().await?;
    let context = repo.context();
    // check if an instance is started
    let status = repo.client.get_instance().await?;
    if !status.name.is_empty() {
        // instance exists
        if status.name != context.name {
            if force {
                // stop instance
                repo.client.stop_instance().await?;
            } else {
                bail!("Currently running challenge");
            }
        }
    } else {
        repo.client.start_instance(&context.name).await?;
    }
    // run exploit script

    // stop instance if necessary

    unimplemented!();
}
