use std::path::Path;
use failure::Error;

use config::GithubConfig;
use {Provider, Repo};

#[derive(Debug, Clone, PartialEq)]
pub struct GitHub {
    cfg: GithubConfig,
}

impl GitHub {
    pub fn with_config(cfg: GithubConfig) -> GitHub {
        GitHub { cfg }
    }
}

impl Provider for GitHub {
    fn name(&self) -> &str {
        "github"
    }

    fn repositories(&self) -> Result<Vec<Repo>, Error> {
        unimplemented!()
    }

    fn download(&self, repo: &Repo, destination: &Path) -> Result<(), Error> {
        unimplemented!()
    }
}
