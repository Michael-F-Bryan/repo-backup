use serde_derive::{Deserialize, Serialize};
use slog::Logger;

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
