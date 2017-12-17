use std::io::Write;
use std::path::Path;
use failure::{Error, ResultExt};
use git2::{AutotagOption, Cred, CredentialType, FetchOptions, FetchPrune, RemoteCallbacks,
           Repository};
use git2::build::RepoBuilder;

use config::Config;
use github::GitHub;
use gitlab_provider::Gitlab;
use {Provider, Repo};

/// A driver for orchestrating the process of fetching a list of repositories
/// and then downloading each of them.
#[derive(Debug, Clone, PartialEq)]
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
        debug!("Updating {}", repo.full_name());
        let dest_dir = self.config.general.dest_dir.join(repo.full_name());

        if dest_dir.exists() {
            fetch_updates(&dest_dir, repo).context("`git fetch` failed")?;
        } else {
            clone_repo(&dest_dir, repo).context("`git clone` failed")?;
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
    debug!("Cloning into {}", dest_dir.display());

    let mut builder = RepoBuilder::new();

    let mut fetch_opts = FetchOptions::new();
    fetch_opts
        .prune(FetchPrune::On)
        .download_tags(AutotagOption::All);

    fetch_opts.remote_callbacks(configure_callbacks(repo));

    builder
        .fetch_options(fetch_opts)
        .clone(&repo.url, dest_dir)?;
    Ok(())
}

/// If we are logging at `trace` level, get a `RemoteCallbacks` which will print
/// a progress message.
fn register_logging_cb<'a>(cb: &mut RemoteCallbacks<'a>, repo: &'a Repo) {
    if log_enabled!(::log::Level::Trace) {
        let repo_name = repo.full_name();

        cb.transfer_progress(move |progress| {
            trace!(
                "{} downloaded {}/{} objects ({} bytes)",
                repo_name,
                progress.received_objects(),
                progress.total_objects(),
                progress.received_bytes()
            );
            true
        });
    }
}

fn cred_callback(
    url: &str,
    username_from_url: Option<&str>,
    allowed_types: CredentialType,
) -> Result<Cred, ::git2::Error> {
    // FIXME: This currently isn't authenticating :(
    debug!("Supplying credentials for {}", url);
    trace!("Allowed types: {:?}", allowed_types);

    if let Some(username) = username_from_url {
        if let Ok(creds) = Cred::username(username) {
            return Ok(creds);
        }
    }

    Cred::ssh_key_from_agent("michael").or_else(|_| Cred::default())
}

fn configure_callbacks(repo: &Repo) -> RemoteCallbacks {
    let mut cb = RemoteCallbacks::new();

    register_logging_cb(&mut cb, repo);
    cb.credentials(cred_callback);

    cb
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

    fetch_opts.remote_callbacks(configure_callbacks(repo));

    remote
        .download(&[], Some(&mut fetch_opts))
        .context("Download failed")?;

    if remote.stats().received_bytes() != 0 {
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

    remote.disconnect();

    // Update the references in the remote's namespace to point to the right
    // commits. This may be needed even if there was no packfile to download,
    // which can happen e.g. when the branches have been changed but all the
    // needed objects are available locally.
    remote.update_tips(None, true, AutotagOption::Unspecified, None)?;

    Ok(())
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
