use std::{
    fs::{self, create_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use config::Config;

use crate::{
    berg,
    models::RepoConfig,
};
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

    let berg = crate::berg::Client::new(server);
    // check server status
    let ctf = berg.get_ctf().await?;
    // write .berg file
    let mut berg_file = File::create(root_dir.join(".berg.toml"))?;
    berg_file.write_all(format!("server = \"{}\"", server).as_bytes())?;
    // write .gitignore file
    let mut gitignore_file = File::create(root_dir.join(".gitignore"))?;
    gitignore_file.write_all(".berg.auth".as_bytes())?;

    // download challenges
    let done_dir = &root_dir.join(".done");
    for (category, challenges) in ctf.challenges_by_category {
        let category_dir = root_dir.join(&category);
        let category_done_dir = done_dir.join(&category);
        create_dir_all(&category_dir)?;
        create_dir_all(&category_done_dir)?;
        for challenge in challenges {
            let category_dir = if challenge.solved_by_player || challenge.solved_by_team {
                &category_done_dir
            } else {
                &category_dir
            };
            let challenge_dir = category_dir.join(&challenge.name);
            berg.create_challenge(&challenge, &challenge_dir).await?;
        }
    }

    // extract challenges into folder structure
    // shc2024
    // |- misc
    // |  |- least-suspicious-bit
    // |     |- chall files here
    // |- pwn

    Ok(())
}

pub async fn sync() -> anyhow::Result<()> {
    // find root dir
    let current_dir = std::env::current_dir().context("could not get current directory")?;
    let root_dir = find_berg_toml_dir(&current_dir)?;
    // get challenges
    let berg_settings = Config::builder()
        .add_source(config::File::from(root_dir.join(".berg.toml")))
        .build()
        .context("could not parse .berg.toml")?
        .try_deserialize::<RepoConfig>()?;

    let mut berg_client = berg::Client::new(&berg_settings.server);
    if let Some(token) = do_auth(&root_dir) {
        berg_client = berg_client.authenticate(&token);
    }

    let ctf = berg_client.get_ctf().await?;
    ctf.challenges_by_category
        .iter()
        .for_each(|(category, challenges)| {
            challenges.iter().for_each(|challenge| {
                let done = challenge.solved_by_player || challenge.solved_by_team;
                let challenge_dir = root_dir.join(category).join(&challenge.name);
                let done_challenge_dir =
                    root_dir.join(".done").join(category).join(&challenge.name);
                if done {
                    if done_challenge_dir.exists() {
                        return;
                    }
                    if challenge_dir.exists() {
                        let _ = fs::rename(challenge_dir, done_challenge_dir);
                    }
                } else if !challenge_dir.exists() {
                    // create challenge dir
                }
            })
        });

    // move done challenges to .done
    // download any new challenges

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

fn find_berg_toml_dir(starting_dir: &Path) -> anyhow::Result<PathBuf> {
    let mut current_dir = starting_dir.to_path_buf();

    loop {
        let berg_toml_path = current_dir.join(".berg.toml");

        if berg_toml_path.exists() {
            return Ok(current_dir);
        }

        // Check if we've reached the root directory
        if !current_dir.pop() {
            break;
        }
    }

    Err(anyhow::anyhow!(
        ".berg.toml not found in any parent directories"
    ))
}
