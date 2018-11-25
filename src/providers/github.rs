use super::Provider;
use crate::config::Config;
use crate::GitRepo;
use failure::{Error, SyncFailure};
use futures::stream::{self, Stream};
use futures::Future;
use hubcaps::Credentials;

/// Retrieve GitHub repositories.
#[derive(Debug, Clone, PartialEq)]
pub struct GitHub {
    cfg: GitHubConfig,
}

impl GitHub {
    pub fn new(cfg: GitHubConfig) -> GitHub {
        GitHub { cfg }
    }

    pub fn from_config(cfg: &Config) -> Result<GitHub, Error> {
        let gh_config = cfg.get_deserialized(GitHubConfig::KEY)?;
        Ok(GitHub::new(gh_config))
    }
}

impl Provider for GitHub {
    fn repositories(&self) -> Box<Stream<Item = GitRepo, Error = Error>> {
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
    use serde::ser::{Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        creds: &Credentials,
        ser: S,
    ) -> Result<S::Ok, S::Error> {
        Shim::from(creds.clone()).serialize(ser)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        de: D,
    ) -> Result<Credentials, D::Error> {
        let shim = Shim::deserialize(de)?;
        Ok(shim.into())
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    enum Shim {
        Token(String),
        Client(String, String),
    }

    impl From<Credentials> for Shim {
        fn from(other: Credentials) -> Shim {
            match other {
                Credentials::Client(l, r) => Shim::Client(l, r),
                Credentials::Token(tok) => Shim::Token(tok),
            }
        }
    }

    impl From<Shim> for Credentials {
        fn from(other: Shim) -> Credentials {
            match other {
                Shim::Client(l, r) => Credentials::Client(l, r),
                Shim::Token(tok) => Credentials::Token(tok),
            }
        }
    }
}
