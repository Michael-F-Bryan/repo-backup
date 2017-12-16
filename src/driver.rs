use failure::{Error, ResultExt};

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

    fn update_repos(&self, repos: &[Repo]) -> Result<(), UpdateOutcome> {
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
            Err(UpdateOutcome { errors })
        }
    }

    fn update_repo(&self, repo: &Repo) -> Result<(), Error> {
        unimplemented!()
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

#[derive(Debug, Fail)]
#[fail(display = "One or more errors ecountered while updating repos")]
struct UpdateOutcome {
    errors: Vec<(Repo, Error)>,
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
