use anyhow::{bail, Context};
use fancy::printcoln;
use tracing::{info, warn};
/// Initialises a challenge repository in the current directory
pub async fn init(
    server: &str,
    path: &Option<String>,
    basic_auth: &Option<String>,
) -> anyhow::Result<()> {
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
        Some(inquire::Password::new("Enter your authentication token: ").prompt()?)
    } else {
        None
    };

    let basic_auth = basic_auth.to_owned().map(|auth| {
        let mut parts = auth.split(':');
        let username = parts.next().unwrap();
        let password = parts.next().unwrap();
        (username.to_owned(), password.to_owned())
    });

    let repo = crate::berg_repo::BergRepo::create(&root_dir, server, &auth_token, &basic_auth)?;
    println!("Repository initialised at {}", root_dir.display());
    repo.sync(true, false).await?;
    println!("Repository synchronised.");
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

pub async fn instance_start(challenge: &str, force: bool) -> anyhow::Result<()> {
    let repo = crate::berg_repo::BergRepo::from_env()?;

    let current_instance = repo.client.get_current_instance().await;
    if let Ok(instance) = current_instance {
        if let Some(name) = &instance.name {
            if name == challenge {
                printcoln!("Already running the same challenge instance.");
                return Ok(());
            } else if force {
                printcoln!("Stopping current instance.");
                repo.client.stop_instance().await?;
            } else {
                bail!("Currently running a different challenge instance.");
            }
        }
    }

    let instance = repo.client.start_instance(challenge).await?;

    printcoln!("Instance started: {:?}", instance);

    Ok(())
}

pub async fn instance_stop() -> anyhow::Result<()> {
    let repo = crate::berg_repo::BergRepo::from_env()?;
    repo.client.stop_instance().await?;

    info!("Instance stopped.");

    Ok(())
}

pub async fn instance_info() -> anyhow::Result<()> {
    let repo = crate::berg_repo::BergRepo::from_env()?;
    let current_instance = repo.client.get_current_instance().await?;
    if let Some(name) = &current_instance.name {
        printcoln!("Challenge instance: {}", name);
        if let Some(services) = &current_instance.services {
            for service in services {
                if let Some(service_name) = &service.name {
                    if let (Some(protocol), Some(hostname)) = (&service.protocol, &service.hostname) {
                        printcoln!(
                            "{}: {}://{}:{}",
                            service_name,
                            protocol,
                            hostname,
                            service.port
                        );
                    }
                } else if let (Some(protocol), Some(hostname)) = (&service.protocol, &service.hostname) {
                    printcoln!(
                        "{}://{}:{}",
                        protocol,
                        hostname,
                        service.port
                    );
                }
            }
        }
    } else {
        printcoln!("No challenge instance running.");
    }
    Ok(())
}

pub async fn instance_exploit(
    script: &str,
    cmd: &str,
    start: bool,
    stop: bool,
    force: bool,
) -> anyhow::Result<()> {
    let repo = crate::berg_repo::BergRepo::from_env()?;
    // check if an instance is started
    let current_instance = repo.client.get_current_instance().await;
    if let Ok(instance) = current_instance {
        if let Some(name) = &instance.name {
            // instance exists
            if name != script { // Use script as challenge name for now
                if force {
                    // stop instance
                    repo.client.stop_instance().await?;
                } else {
                    bail!("Currently running challenge");
                }
            }
        }
    } else {
        repo.client.start_instance(script).await?; // Use script as challenge name for now
    }
    // run exploit script

    // stop instance if necessary

    unimplemented!();
}
