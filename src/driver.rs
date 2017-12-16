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
        info!("Starting repo-backup");

        let providers = get_providers(&self.config)?;
        let repos = self.get_repos_from_providers(&providers)?;

        Ok(())
    }

    fn get_repos_from_providers(&self, providers: &[Box<Provider>]) -> Result<Vec<Repo>, Error> {
        let mut repos = Vec::new();

        for provider in providers {
            info!("Fetching repositories from {}", provider.name());
            let found = provider
                .repositories()
                .context("Unable to fetch repositories")?;

            debug!("Found {} repos from {}", found.len(), provider.name());
            repos.extend(found);
        }

        Ok(repos)

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
