use std::fmt::{self, Debug, Formatter};
use failure::{Error, ResultExt};

use config::GithubConfig;
use utils::Paginated;
use {Provider, Repo};

#[derive(Clone)]
pub struct GitHub {
    cfg: GithubConfig,
}

impl GitHub {
    pub fn with_config(cfg: GithubConfig) -> GitHub {
        GitHub { cfg }
    }

    fn get_owned(&self) -> Result<Vec<Repo>, Error> {
        debug!("Fetching owned repositories");

        let mut owned = Vec::new();

        for repo in Paginated::new(&self.cfg.api_key, "https://api.github.com/user/repos") {
            let repo: RawRepo = repo?;
            owned.push(Repo::from(repo));
        }

        debug!("{} owned repos", owned.len());
        Ok(owned)
    }

    fn get_starred(&self) -> Result<Vec<Repo>, Error> {
        debug!("Fetching starred repositories");

        let mut starred = Vec::new();

        for repo in Paginated::new(&self.cfg.api_key, "https://api.github.com/user/starred") {
            let repo: RawRepo = repo?;
            starred.push(Repo::from(repo));
        }

        debug!("{} starred repos", starred.len());
        Ok(starred)
    }
}

impl Provider for GitHub {
    fn name(&self) -> &str {
        "github"
    }

    fn repositories(&self) -> Result<Vec<Repo>, Error> {
        let mut repos = Vec::new();

        if self.cfg.owned {
            repos.extend(self.get_owned()
                .context("Unable to fetch owned repositories")?);
        }
        if self.cfg.starred {
            repos.extend(self.get_starred()
                .context("Unable to fetch starred repositories")?);
        }

        Ok(repos)
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
    clone_url: String,
    owner: Owner,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct Owner {
    login: String,
    #[serde(rename = "type")] kind: String,
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
