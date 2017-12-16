use std::fmt::{self, Debug, Formatter};
use failure::{Error, ResultExt};

use config::GithubConfig;
use utils::Paginated;
use {Provider, Repo};

#[derive(Clone)]
pub struct GitHub {
    api_key: String,
}

impl GitHub {
    pub fn with_config(cfg: GithubConfig) -> GitHub {
        GitHub {
            api_key: cfg.api_key,
        }
    }
}

impl Provider for GitHub {
    fn name(&self) -> &str {
        "github"
    }

    fn repositories(&self) -> Result<Vec<Repo>, Error> {
        let mut repos = Vec::new();

        debug!("Fetching owned repositories");

        let owned = Paginated::new(&self.api_key, "https://api.github.com/user/repos")
            .collect::<Result<Vec<RawRepo>, Error>>()
            .context("Unable to fetch owned repositories")?;

        debug!("{} owned repos", owned.len());
        repos.extend(owned.into_iter().map(Into::into));

        let starred = Paginated::new(&self.api_key, "https://api.github.com/user/starred")
            .collect::<Result<Vec<RawRepo>, Error>>()
            .context("Unable to fetch starred repositories")?;

        debug!("{} starred repos", starred.len());
        repos.extend(starred.into_iter().map(Into::into));

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
