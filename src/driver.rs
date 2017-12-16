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

        if providers.is_empty() {
            warn!("No providers found");
        }

        for provider in &providers {
            info!("Fetching repositories from {}", provider.name());
            let repos = provider
                .repositories()
                .context("Unable to fetch repositories")?;
        }

        Ok(())
    }
}

fn get_providers(cfg: &Config) -> Result<Vec<Box<Provider>>, Error> {
    let mut providers: Vec<Box<Provider>> = Vec::new();

    if let Some(gh_config) = cfg.github.as_ref() {
        let gh = GitHub::with_config(gh_config.clone())?;
        providers.push(Box::new(gh));
    }

    Ok(providers)
}
