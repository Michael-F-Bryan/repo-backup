use std::path::PathBuf;
use github_rs::client::Github;
use github_rs::headers;
use serde_json::value::from_value;
use log::LogLevel::Debug;

use errors::*;
use data::{Repo, Summary, Paginated};


pub struct Client {
    backup_dir: PathBuf,
    inner: Github,
}

impl Client {
    /// Create a new client using the provided arguments.
    pub fn new<S, P>(token: S, backup_dir: P) -> Result<Client>
    where
        S: AsRef<str>,
        P: Into<PathBuf>,
    {
        let inner = Github::new(token).chain_err(
            || "Couldn't create a GitHub client",
        )?;

        Ok(Client {
            inner: inner,
            backup_dir: backup_dir.into(),
        })
    }

    pub fn run(&self) -> Result<Summary> {
        info!("Searching GitHub for repositories");
        let owned = self.get_owned_repositories()?;

        unimplemented!()
    }

    fn get_owned_repositories(&self) -> Result<Vec<Repo>> {
        let paged: Paginated<Vec<Repo>> = Paginated::new(&self.inner, "user/repos");

        let mut owned_repos = Vec::new();

        for page in paged {
            let repos = page?;
            owned_repos.extend(repos);
        }

        debug!("Found {} owned repos", owned_repos.len());

        Ok(owned_repos)
    }
}
