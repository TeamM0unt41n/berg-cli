use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::{bail, Context};
use config::Config;
use flate2::bufread::GzDecoder;
use tar::Archive;
use tracing::{info, warn};

use crate::{
    berg,
    models::{Challenge, RepoConfig, SubmitFlagResult},
};

#[derive(Clone)]
pub struct BergRepo {
    pub(crate) client: berg::Client,
    config: RepoConfig,
    path: PathBuf,
}

impl BergRepo {
    /// Tries to open the repository at the current directory or any of its parents
    pub fn from_env() -> anyhow::Result<Self> {
        let current_dir = std::env::current_dir().context("could not get current directory")?;
        let root_dir = find_berg_toml_dir(&current_dir)?;
        Self::open(&root_dir)
    }

    pub fn create(dir: &PathBuf, server: &str, token: &Option<String>) -> anyhow::Result<Self> {
        let mut berg = crate::berg::Client::new(server);
        if let Some(token) = token {
            berg = berg.authenticate(token);
        }

        let config = RepoConfig {
            server: server.to_owned(),
        };

        let mut repo = Self {
            client: berg,
            config,
            path: dir.to_owned(),
        };

        repo.create_initial_structure(token)?;
        Ok(repo)
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

    pub fn create_initial_structure(&mut self, token: &Option<String>) -> anyhow::Result<()> {
        if self.path.exists() && self.path.read_dir()?.next().is_some() {
            bail!("Directory is not empty.");
        }

        fs::create_dir_all(&self.path)?;

        // initialise git repo with main branch
        let repo = git2::Repository::init(&self.path)?;
        let mut branch = repo.find_branch("master", git2::BranchType::Local)?;
        branch.rename("main", false)?;

        let done_dir = self.done_path();
        fs::create_dir_all(&done_dir)?;
        fs::create_dir_all(&self.berg_dir())?;
        let mut gitignore_file = File::create(self.path.join(".gitignore"))?;
        gitignore_file.write_all(".berg/auth".as_bytes())?;
        let mut berg_file = File::create(self.config_path())?;
        berg_file.write_all(toml::to_string(&self.config)?.as_bytes())?;

        if let Some(token) = token {
            let mut auth_file = File::create(self.auth_path())?;
            auth_file.write_all(token.as_bytes())?;
        }
        tracing::info!("Repository created at {}", self.path.display());

        Ok(())
    }

    pub async fn sync(&self, force: bool, flagdump: bool) -> anyhow::Result<()> {
        // if repo has uncommited changes, abort
        let repo = git2::Repository::open(&self.path)?;
        let mut status_opts = git2::StatusOptions::new();
        status_opts.include_untracked(true);
        let statuses = repo.statuses(Some(&mut status_opts))?;
        if !force && !statuses.is_empty() {
            bail!("Repository has uncommitted changes.");
        }

        tracing::debug!("loading ctf");
        let ctf = self.client.get_ctf().await?;

        let mut tried_flags: HashMap<String, HashSet<String>> = self.load_tried_flags()?;
        tracing::debug!(
            "loaded {} tried flags",
            tried_flags.iter().map(|(_, v)| v.len()).sum::<usize>()
        );

        for (category, challenges) in ctf.challenges_by_category {
            tracing::debug!("syncing category {}", category);
            let category_dir = self.path.join(&category);
            let category_done_dir = self.done_path().join(&category);
            fs::create_dir_all(&category_dir)?;
            fs::create_dir_all(&category_done_dir)?;
            for challenge in challenges {
                tracing::debug!("syncing challenge {}", challenge.name);
                let done = challenge.solved_by_player || challenge.solved_by_team;
                let challenge_dir = category_dir.join(&challenge.name);
                let done_challenge_dir = category_done_dir.join(&challenge.name);
                if done {
                    if done_challenge_dir.exists() {
                        continue;
                    }
                    if challenge_dir.exists() {
                        let _ = fs::rename(challenge_dir, &done_challenge_dir);
                    }
                    // neither exist, download challenge to done
                    self.create_challenge(&challenge, &done_challenge_dir)
                        .await?;
                } else if !challenge_dir.exists() {
                    // create challenge dir
                    self.create_challenge(&challenge, &challenge_dir).await?;
                } else if flagdump {
                    // challenge dir exists, not done
                    let flag_file = challenge_dir.join(".flag");
                    if flag_file.exists() {
                        // if flag is not in tried flags, try it
                        // if flag is correct, mark challenge as done
                        // either way, add to tried flags
                        tracing::debug!("possible flag for challenge {}", challenge.name);
                        let flag = fs::read_to_string(&flag_file)?;
                        if !tried_flags
                            .get(&challenge.name)
                            .map(|flags| flags.contains(&flag))
                            .unwrap_or(false)
                        {
                            tracing::info!("submitting flag for challenge {}", challenge.name);
                            let flag_result =
                                self.client.submit_flag(&challenge.name, &flag).await?;
                            match flag_result {
                                SubmitFlagResult::Correct => {
                                    tried_flags
                                        .entry(challenge.name.clone())
                                        .or_insert_with(HashSet::new)
                                        .insert(flag);
                                    tracing::info!(
                                        "challenge {} solved, moving to done",
                                        challenge.name
                                    );
                                    let _ = fs::rename(&challenge_dir, &done_challenge_dir);
                                }
                                SubmitFlagResult::Incorrect => {
                                    tracing::info!(
                                        "incorrect flag for challenge {}",
                                        challenge.name
                                    );
                                    tried_flags
                                        .entry(challenge.name.clone())
                                        .or_insert_with(HashSet::new)
                                        .insert(flag);
                                }
                                _ => {
                                    // warn
                                    warn!("Unexpected flag result: {:?}", flag_result);
                                }
                            }
                        }
                    }
                }
            }
        }

        self.save_tried_flags(&tried_flags)?;

        Ok(())
    }

    fn try_auth(&mut self) -> anyhow::Result<()> {
        let auth_file = self.auth_path();
        if auth_file.exists() {
            let token = fs::read_to_string(&auth_file)?;
            self.client = self.client.authenticate(&token);
        }
        Ok(())
    }

    pub fn authenticate(&mut self, token: &str) -> anyhow::Result<()> {
        let client = self.client.authenticate(token);
        // write authentication token to file
        let mut file = File::create(self.auth_path())?;
        file.write_all(token.as_bytes())?;
        self.client = client;
        Ok(())
    }

    fn load_tried_flags(&self) -> anyhow::Result<HashMap<String, HashSet<String>>> {
        let tried_flags_file = self.berg_dir().join("tried_flags");
        if tried_flags_file.exists() {
            serde_json::from_reader(File::open(tried_flags_file)?)
                .context("could not parse tried_flags file")
        } else {
            Ok(HashMap::new())
        }
    }

    fn save_tried_flags(
        &self,
        tried_flags: &HashMap<String, HashSet<String>>,
    ) -> anyhow::Result<()> {
        let tried_flags_file = self.berg_dir().join("tried_flags");
        serde_json::to_writer(File::create(tried_flags_file)?, tried_flags)
            .context("could not write tried_flags file")
    }

    pub async fn submit_flag(
        &self,
        challenge: &str,
        flag: &str,
    ) -> anyhow::Result<SubmitFlagResult> {
        let mut tried_flags = self.load_tried_flags()?;
        if tried_flags
            .get(challenge)
            .map(|flags| flags.contains(flag))
            .unwrap_or(false)
        {
            return Ok(SubmitFlagResult::Incorrect);
        }
        let flag_result = self.client.submit_flag(challenge, flag).await?;
        match flag_result {
            SubmitFlagResult::Correct => {
                tried_flags
                    .entry(challenge.to_string())
                    .or_insert_with(HashSet::new)
                    .insert(flag.to_string());
            }
            SubmitFlagResult::Incorrect => {
                tried_flags
                    .entry(challenge.to_string())
                    .or_insert_with(HashSet::new)
                    .insert(flag.to_string());
            }
            _ => {
                // warn
                warn!("Unexpected flag result: {:?}", flag_result);
            }
        }
        self.save_tried_flags(&tried_flags)?;
        Ok(flag_result)
    }
}

impl BergRepo {
    pub async fn create_challenge(
        &self,
        challenge: &Challenge,
        challenge_dir: &PathBuf,
    ) -> anyhow::Result<()> {
        fs::create_dir_all(challenge_dir)?;

        // readme file
        let readme_file = challenge_dir.join("README.md");
        let mut readme_file = File::create(&readme_file)?;
        let description = html2md::parse_html(&challenge.description);
        let readme_content = format!(
            r"# {}

By **{}**

## Description

{}

",
            &challenge.name, &challenge.author, &description
        );
        readme_file.write_all(readme_content.as_bytes())?;

        info!("created challenge {}", &challenge.name);
        for attachment in &challenge.attachments {
            let url = self.client.server_url().join(&attachment.download_url)?;
            let file: bytes::Bytes = reqwest::get(url).await?.bytes().await?;
            info!("grabbed attachment {}", &attachment.file_name);
            if attachment.file_name.ends_with(".tar.gz") {
                if untar_file(file, challenge, challenge_dir).is_err() {
                    info!(
                        "could not extract supposed archive in challenge {}: {}",
                        &challenge.name, attachment.file_name
                    );
                }
            } else {
                // download file normally
                info!(
                    "non conformant file found in challenge {}: {}",
                    &challenge.name, attachment.file_name
                );
            }
        }
        Ok(())
    }
}

fn untar_file(
    file: bytes::Bytes,
    challenge: &Challenge,
    challenge_dir: &PathBuf,
) -> anyhow::Result<()> {
    let tar = GzDecoder::new(&file[..]);
    let mut archive = Archive::new(tar);
    if archive
        .entries()?
        .flatten()
        .flat_map(|e| e.path().map(|e| e.into_owned()))
        .all(|e| e.starts_with(&challenge.name))
    {
        // extract into parent dir
        let tar = GzDecoder::new(&file[..]);
        let mut archive = Archive::new(tar);
        archive.unpack(challenge_dir.parent().unwrap())?;
    } else {
        // extract into dir
        let tar = GzDecoder::new(&file[..]);
        let mut archive = Archive::new(tar);
        archive.unpack(challenge_dir)?;
    }
    Ok(())
}

// paths
impl BergRepo {
    fn done_path(&self) -> PathBuf {
        self.path.join(".done")
    }

    fn auth_path(&self) -> PathBuf {
        self.berg_dir().join("auth")
    }

    fn config_path(&self) -> PathBuf {
        self.path.join(".berg.toml")
    }

    fn berg_dir(&self) -> PathBuf {
        self.path.join(".berg")
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

fn find_berg_toml_dir(starting_dir: &PathBuf) -> anyhow::Result<PathBuf> {
    let mut current_dir = starting_dir.to_owned();

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
