use std::io::Write;
use std::path::Path;
use failure::{Error, ResultExt};
use git2::{AutotagOption, FetchOptions, FetchPrune, Repository};
use git2::build::RepoBuilder;

use config::Config;
use github::GitHub;
use {Provider, Repo};

#[derive(Debug, Clone, PartialEq)]
pub struct Driver {
    config: Config,
}

impl Driver {
    pub fn with_config(config: Config) -> Driver {
        Driver { config }
    }

    pub fn run(&self) -> Result<(), Error> {
        let providers = get_providers(&self.config)?;
        let repos = self.get_repos_from_providers(&providers)?;
        self.update_repos(&repos)?;

        Ok(())
    }

    fn update_repos(&self, repos: &[Repo]) -> Result<(), UpdateFailure> {
        info!("Updating repositories");
        let mut errors = Vec::new();

        for repo in repos {
            if let Err(e) = self.update_repo(repo) {
                warn!("Updating {} failed, {}", repo.name, e);
                errors.push((repo.clone(), e));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(UpdateFailure { errors })
        }
    }

    fn update_repo(&self, repo: &Repo) -> Result<(), Error> {
        debug!("Updating {}", repo.full_name());
        let dest_dir = self.config.general.dest_dir.join(repo.full_name());

        if dest_dir.exists() {
            fetch_updates(&dest_dir, repo).context("`git fetch` failed")?;
        } else {
            clone_repo(&dest_dir, repo).context("`git clone` failed")?;
        }

        Ok(())
    }

    fn get_repos_from_providers(&self, providers: &[Box<Provider>]) -> Result<Vec<Repo>, Error> {
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
    debug!("Cloning into {}", dest_dir.display());

    RepoBuilder::new().clone(&repo.url, dest_dir)?;
    Ok(())
}

fn fetch_updates(dest_dir: &Path, repo: &Repo) -> Result<(), Error> {
    let git_repo = Repository::open(dest_dir).context("Not a git repository")?;
    let mut remote = git_repo
        .find_remote("origin")
        .or_else(|_| git_repo.remote_anonymous("origin"))
        .context("The repo has no `origin` remote")?;

    let mut fetch_opts = FetchOptions::default();
    fetch_opts
        .prune(FetchPrune::On)
        .download_tags(AutotagOption::All);
    remote
        .download(&[], Some(&mut fetch_opts))
        .context("Download failed")?;

    if remote.stats().received_bytes() != 0 {
        // If there are local objects (we got a thin pack), then tell the user
        // how many objects we saved from having to cross the network.
        let stats = remote.stats();
        if stats.local_objects() > 0 {
            debug!(
                "Received {} objects in {} bytes for {} (used {} local \
                 objects)",
                stats.indexed_objects(),
                stats.received_bytes(),
                repo.full_name(),
                stats.local_objects()
            );
        } else {
            debug!(
                "Received {} objects in {} bytes for {}",
                stats.indexed_objects(),
                stats.received_bytes(),
                repo.full_name()
            );
        }
    }

    // Disconnect the underlying connection to prevent from idling.
    remote.disconnect();

    // Update the references in the remote's namespace to point to the right
    // commits. This may be needed even if there was no packfile to download,
    // which can happen e.g. when the branches have been changed but all the
    // needed objects are available locally.
    remote.update_tips(None, true, AutotagOption::Unspecified, None)?;

    Ok(())
}

#[derive(Debug, Fail)]
#[fail(display = "One or more errors ecountered while updating repos")]
pub struct UpdateFailure {
    errors: Vec<(Repo, Error)>,
}

impl UpdateFailure {
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

    if providers.is_empty() {
        warn!("No providers found");
    }

    Ok(providers)
}
