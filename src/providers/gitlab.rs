use crate::git::GitRepo;
use crate::providers::Provider;
use failure::{Error, Fail, SyncFailure};
use futures::sync::mpsc;
use futures::Stream;
use serde_derive::{Deserialize, Serialize};
use slog::Logger;
use std::path::Path;
use std::thread;

const DEFAULT_HOSTNAME: &str = "gitlab.com";

/// The GitLab provider.
#[derive(Debug, Clone)]
pub struct GitLab {
    cfg: GitLabConfig,
    logger: Logger,
}

impl GitLab {
    pub fn new(cfg: GitLabConfig, logger: Logger) -> GitLab {
        GitLab { cfg, logger }
    }
}

impl Provider for GitLab {
    fn repositories(&self) -> Box<Stream<Item = GitRepo, Error = Error>> {
        let (tx, rx) = mpsc::unbounded();
        let cfg = self.cfg.clone();
        let logger = self.logger.clone();

        thread::spawn(move || {
            spawn_client(cfg, tx, &logger);
            debug!(logger, "Finished fetching GitLab repos");
        });

        Box::new(
            rx.map_err(|_| failure::err_msg("Unable to read from the channel"))
                .and_then(|item| item),
        )
    }
}

fn spawn_client(
    cfg: GitLabConfig,
    tx: mpsc::UnboundedSender<Result<GitRepo, Error>>,
    logger: &Logger,
) {
    debug!(logger, "Creating the GitLab client");

    let client = match gitlab::Gitlab::new(cfg.hostname, cfg.api_key) {
        Ok(c) => c,
        Err(e) => {
            let err = SyncFailure::new(e)
                .context("Unable to create the GitLab client");
            let _ = tx.unbounded_send(Err(err.into()));
            return;
        }
    };

    debug!(logger, "Fetching GitLab projects");

    let projects = match client.owned_projects() {
        Ok(p) => p,
        Err(e) => {
            warn!(logger, "Unable to fetch the project list";
                "error" => e.to_string());
            let err =
                SyncFailure::new(e).context("Unable to fetch the project list");
            let _ = tx.unbounded_send(Err(err.into()));
            return;
        }
    };

    debug!(logger, "Retreived the project list";
        "project-count" => projects.len());

    for project in projects {
        trace!(logger, "Found project";
            "name" => &project.name_with_namespace,
            "ssh-url" => &project.ssh_url_to_repo,
            "description" => project.description.as_ref(),
            "stars" => project.star_count,
            "forks" => project.forks_count,
            "stats" => project.statistics.as_ref().map(|stats| format!("{:?}", stats)));

        let repo = project_to_repo(project);
        if tx.unbounded_send(Ok(repo)).is_err() {
            // the receiver was dropped so there's no point continuing...
            return;
        }
    }
}

fn project_to_repo(project: gitlab::Project) -> GitRepo {
    GitRepo {
        dest_dir: Path::new("gitlab")
            .join(project.namespace.path)
            .join(project.path),
        ssh_url: project.ssh_url_to_repo,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct GitLabConfig {
    #[serde(default = "default_hostname")]
    pub hostname: String,
    pub api_key: String,
}

fn default_hostname() -> String {
    DEFAULT_HOSTNAME.to_string()
}

impl Default for GitLabConfig {
    fn default() -> GitLabConfig {
        GitLabConfig {
            hostname: default_hostname(),
            api_key: String::new(),
        }
    }
}
