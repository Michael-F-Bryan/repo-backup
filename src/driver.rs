use std::io::Write;
use std::path::Path;
use failure::{Error, ResultExt};

use config::Config;
use github::GitHub;
use gitlab_provider::Gitlab;
use {Provider, Repo};

/// A driver for orchestrating the process of fetching a list of repositories
/// and then downloading each of them.
#[derive(Debug, Clone)]
pub struct Driver {
    config: Config,
}

impl Driver {
    /// Create a new `Driver` with the provided config.
    pub fn with_config(config: Config) -> Driver {
        Driver { config }
    }

    /// Download a list of all repositories from the `Provider`s found in the
    /// configuration file, then fetch any recent changes (running `git clone`
    /// if necessary).
    pub fn run(&self) -> Result<(), Error> {
        info!("Starting repository backup");

        let providers = get_providers(&self.config)?;
        let repos = self.get_repos_from_providers(&providers)?;
        self.update_repos(&repos)?;

        info!("Finished repository backup");
        Ok(())
    }

    /// Update the provided repositories.
    pub fn update_repos(&self, repos: &[Repo]) -> Result<(), UpdateFailure> {
        info!("Updating repositories");
        let mut errors = Vec::new();

        for repo in repos {
            if let Err(e) = self.update_repo(repo) {
                warn!("Updating {} failed, {}", repo.name, e);
                errors.push((repo.clone(), e));
            }

            if errors.len() >= 10 {
                error!("Too many errors, bailing...");
                break;
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(UpdateFailure { errors })
        }
    }

    fn update_repo(&self, repo: &Repo) -> Result<(), Error> {
        let dest_dir = self.config.general.dest_dir.join(repo.full_name());

        if dest_dir.exists() {
            debug!("Fetching updates for {}", repo.full_name());
            fetch_updates(&dest_dir)?;
        } else {
            debug!("Cloning into {}", dest_dir.display());
            clone_repo(&dest_dir, repo)?;
        }

        Ok(())
    }

    /// Iterate over the `Provider`s and collect all the repositories they've
    /// found into one big list.
    pub fn get_repos_from_providers(
        &self,
        providers: &[Box<Provider>],
    ) -> Result<Vec<Repo>, Error> {
        let mut repos = Vec::new();

        for provider in providers {
            info!("Fetching repositories from {}", provider.name());
            let found = provider
                .repositories()
                .context("Unable to fetch repositories")?;

            info!("Found {} repos from {}", found.len(), provider.name());
            repos.extend(found);
        }

        Ok(repos)
    }
}

fn clone_repo(dest_dir: &Path, repo: &Repo) -> Result<(), Error> {
    cmd!("git clone --quiet {} {}", &repo.url, dest_dir.display())
}

fn fetch_updates(dest_dir: &Path) -> Result<(), Error> {
    cmd!(in dest_dir; "git pull --ff-only --prune --quiet --recurse-submodules")
}

/// A wrapper around one or more failures during the updating process.
#[derive(Debug, Fail)]
#[fail(display = "One or more errors ecountered while updating repos")]
pub struct UpdateFailure {
    errors: Vec<(Repo, Error)>,
}

impl UpdateFailure {
    /// Print a "backtrace" for each error encountered.
    pub fn display<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writeln!(
            writer,
            "There were {} errors updating repositories",
            self.errors.len()
        )?;

        for &(ref repo, ref err) in &self.errors {
            writeln!(writer, "Error: {} failed with {}", repo.full_name(), err)?;
            for cause in err.causes().skip(1) {
                writeln!(writer, "\tCaused By: {}", cause)?;
            }
        }

        Ok(())
    }
}

fn get_providers(cfg: &Config) -> Result<Vec<Box<Provider>>, Error> {
    let mut providers: Vec<Box<Provider>> = Vec::new();

    if let Some(gh_config) = cfg.github.as_ref() {
        let gh = GitHub::with_config(gh_config.clone());
        providers.push(Box::new(gh));
    }

    if let Some(gl_config) = cfg.gitlab.as_ref() {
        let gl = Gitlab::with_config(gl_config.clone())?;
        providers.push(Box::new(gl));
    }

    if providers.is_empty() {
        warn!("No providers found");
    }

    Ok(providers)
}
