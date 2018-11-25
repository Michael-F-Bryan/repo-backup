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

        thread::spawn(move || {
            spawn_client(cfg, tx);
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
) {
    let client = match gitlab::Gitlab::new(cfg.hostname, cfg.api_key) {
        Ok(c) => c,
        Err(e) => {
            let err = SyncFailure::new(e)
                .context("Unable to create the gitlab client");
            let _ = tx.unbounded_send(Err(err.into()));
            return;
        }
    };

    let projects = match client.projects() {
        Ok(p) => p,
        Err(e) => {
            let err =
                SyncFailure::new(e).context("Unable to fetch the project list");
            let _ = tx.unbounded_send(Err(err.into()));
            return;
        }
    };

    for project in projects {
        let repo = project_to_repo(project);
        if tx.unbounded_send(Ok(repo)).is_err() {
            // the receiver was dropped so there's no point continuing...
            return;
        }
    }
}

fn project_to_repo(project: gitlab::Project) -> GitRepo {
    GitRepo {
        dest_dir: Path::new("gitlab").join(project.name_with_namespace),
        ssh_url: project.ssh_url_to_repo,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct GitLabConfig {
    pub hostname: String,
    pub api_key: String,
}

impl Default for GitLabConfig {
    fn default() -> GitLabConfig {
        GitLabConfig {
            hostname: DEFAULT_HOSTNAME.to_string(),
            api_key: String::new(),
        }
    }
}
