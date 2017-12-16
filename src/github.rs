use std::path::Path;
use std::fmt::{self, Debug, Formatter};
use github_rs::client::{Executor, Github as GhClient};
use failure::{Error, ResultExt};

use config::GithubConfig;
use {Provider, Repo, SyncResult};

#[derive(Clone)]
pub struct GitHub {
    client: GhClient,
}

impl GitHub {
    pub fn with_config(cfg: GithubConfig) -> Result<GitHub, Error> {
        let client = GhClient::new(&cfg.api_key)
            .sync()
            .context("Invalid API token")?;

        Ok(GitHub { client })
    }
}

impl Provider for GitHub {
    fn name(&self) -> &str {
        "github"
    }

    fn repositories(&self) -> Result<Vec<Repo>, Error> {
        let mut repos = Vec::new();

        debug!("Fetching owned repositories");
        let (headers, status, got) = self.client
            .get()
            .user()
            .repos()
            .execute::<Vec<RawRepo>>()
            .sync()?;

        if log_enabled!(::log::Level::Trace) {
            trace!("Status Code: {}, {:?}", status.as_u16(), status);
            for line in format!("Headers {:#?}", headers).lines() {
                trace!("{}", line);
            }
        }

        if let Some(owned) = got {
            debug!("Found {} owned repos", owned.len());
            repos.extend(owned.into_iter().map(Into::into));
        } else {
            debug!("No owned repos found");
        }

        Ok(repos)
    }

    fn download(&self, repo: &Repo, destination: &Path) -> Result<(), Error> {
        unimplemented!()
    }
}

impl Debug for GitHub {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("GitHub").finish()
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct RawRepo {
    name: String,
    full_name: String,
    description: Option<String>,
    clone_url: String,
    owner: Owner,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct Owner {
    login: String,
    #[serde(rename = "type")]
    kind: String,
}

impl From<RawRepo> for Repo {
    fn from(other: RawRepo) -> Repo {
        Repo { 
            name: other.name,
            owner: other.owner.login,
            url: other.clone_url,
        }
    }
}