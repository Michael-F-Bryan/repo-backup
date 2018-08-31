use failure::{Error, ResultExt};
use gitlab::{Gitlab as Client, Project};

use config::GitLabConfig;
use {Provider, Repo, SyncResult};

/// A provider which queries the GitLab API.
#[derive(Debug)]
pub struct GitLab {
    client: Client,
    cfg: GitLabConfig,
}

impl GitLab {
    /// Create a new `GitLab` provider using its config.
    pub fn with_config(cfg: GitLabConfig) -> Result<GitLab, Error> {
        let client = Client::new(&cfg.url, cfg.api_key.reveal_str())
            .sync()
            .context("Invalid API key")?;

        Ok(GitLab { client, cfg })
    }

    fn get_owned(&self) -> Result<Vec<Repo>, Error> {
        debug!("Fetching owned repos");
        let owned = self.client.owned_projects().sync()?;

        let repos: Vec<Repo> = owned
            .into_iter()
            .map(|p| self.gitlab_project_to_repo(p))
            .collect();

        debug!("Found {} owned projects", repos.len());

        Ok(repos)
    }

    fn get_organisation_repos(&self) -> Result<Vec<Repo>, Error> {
        debug!("Fetching organisation repos");

        let current_user = self
            .client
            .current_user()
            .sync()
            .context("Unable to get the name of the current user")?
            .username;
        trace!("Current GitLab user is {}", current_user);

        let all_repos = self.client.projects().sync()?;

        let org_repos: Vec<Repo> = all_repos
            .into_iter()
            .map(|p| self.gitlab_project_to_repo(p))
            .filter(|r| r.owner != current_user)
            .collect();

        debug!(
            "Found {} repos owned by organisations you are a part of",
            org_repos.len()
        );
        Ok(org_repos)
    }

    fn gitlab_project_to_repo(&self, project: Project) -> Repo {
        let mut split = project.path_with_namespace.split("/");
        let owner = split.next().expect("Namespaces always have an owner");
        let name = split.next().expect("unreachable");

        Repo {
            owner: owner.to_string(),
            name: name.to_string(),
            url: project.ssh_url_to_repo,
            provider: self.name().to_string(),
        }
    }
}

impl Provider for GitLab {
    fn name(&self) -> &str {
        "gitlab"
    }

    fn repositories(&self) -> Result<Vec<Repo>, Error> {
        let mut repos = Vec::new();

        if self.cfg.owned {
            let owned =
                self.get_owned().context("Unable to get owned repos")?;
            repos.extend(owned);
        }

        if self.cfg.organisations {
            let org_repos = self.get_organisation_repos().context(
                "Unable to get repos owned by organisations you are a part of",
            )?;
            repos.extend(org_repos);
        }

        Ok(repos)
    }
}
