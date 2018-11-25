use super::Provider;
use crate::config::Config;
use crate::GitRepo;
use failure::{Error, SyncFailure};
use futures::stream::{self, Stream};
use futures::Future;
use hubcaps::Credentials;
use slog::Logger;

/// Retrieve GitHub repositories.
#[derive(Debug, Clone)]
pub struct GitHub {
    cfg: GitHubConfig,
    logger: Logger,
}

impl GitHub {
    pub fn new(cfg: GitHubConfig, logger: Logger) -> GitHub {
        GitHub { cfg, logger }
    }

    pub fn from_config(cfg: &Config, logger: &Logger) -> Result<GitHub, Error> {
        let gh_config = cfg.get_deserialized(GitHubConfig::KEY)?;
        Ok(GitHub::new(gh_config, logger.clone()))
    }
}

impl Provider for GitHub {
    fn repositories(&self) -> Box<Stream<Item = GitRepo, Error = Error>> {
        debug!(self.logger, "Creating the GitHub client");
        let client = hubcaps::Github::new(
            self.cfg.agent.clone(),
            self.cfg.credentials.clone(),
        );

        let user_repos =
            client.repos().iter(&Default::default()).map(GitRepo::from);

        if self.cfg.orgs {
            Box::new(
                user_repos
                    .select(org_repos(&client))
                    .map_err(SyncFailure::new)
                    .map_err(Error::from),
            )
        } else {
            Box::new(user_repos.map_err(SyncFailure::new).map_err(Error::from))
        }
    }
}

fn org_repos<T>(
    client: &hubcaps::Github<T>,
) -> impl Stream<Item = GitRepo, Error = hubcaps::Error>
where
    T: Clone + hyper::client::connect::Connect,
{
    let c2 = client.clone();
    client
        .orgs()
        .list()
        .map(|orgs| {
            stream::iter_ok::<_, hubcaps::Error>(
                orgs.into_iter().map(|org| org.login),
            )
        }).flatten_stream()
        .map(move |org| c2.org_repos(org).iter(&Default::default()))
        .flatten()
        .map(GitRepo::from)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// The user-agent to use.
    #[serde(default)]
    pub agent: String,
    /// Should we include starred repositories?
    #[serde(default)]
    pub starred: bool,
    /// Should we include repositories from organisations you belong to?
    #[serde(default)]
    pub orgs: bool,
    #[serde(with = "cred_serde_shim")]
    pub credentials: Credentials,
}

impl GitHubConfig {
    pub const KEY: &'static str = "github";
    pub const DEFAULT_AGENT: &'static str = "repo-backup";
}

impl Default for GitHubConfig {
    fn default() -> GitHubConfig {
        GitHubConfig {
            agent: GitHubConfig::DEFAULT_AGENT.into(),
            credentials: Credentials::Token(String::new()),
            starred: true,
            orgs: true,
        }
    }
}

mod cred_serde_shim {
    use super::*;
    use serde::de::{Deserialize, Deserializer};
    use serde::ser::{Error, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        creds: &Credentials,
        ser: S,
    ) -> Result<S::Ok, S::Error> {
        match *creds {
            Credentials::Token(ref token) => token.serialize(ser),
            _ => Err(S::Error::custom("Unknown credentials type")),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        de: D,
    ) -> Result<Credentials, D::Error> {
        let api_key = String::deserialize(de)?;
        Ok(Credentials::Token(api_key))
    }
}
