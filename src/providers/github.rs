use super::Provider;
use crate::config::Config;
use crate::GitRepo;
use failure::{Error, SyncFailure};
use futures::stream::Stream;
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

        Box::new(
            client
                .repos()
                .iter(&Default::default())
                .map(Into::into)
                .map_err(SyncFailure::new)
                .map_err(Error::from),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GitHubConfig {
    pub agent: String,
    #[serde(with = "cred_serde_shim")]
    pub credentials: Option<Credentials>,
}

impl GitHubConfig {
    pub const KEY: &'static str = "github";
    pub const DEFAULT_AGENT: &'static str = "repo-backup";
}

impl Default for GitHubConfig {
    fn default() -> GitHubConfig {
        GitHubConfig {
            agent: GitHubConfig::DEFAULT_AGENT.into(),
            credentials: None,
        }
    }
}

mod cred_serde_shim {
    use super::*;
    use serde::de::{Deserialize, Deserializer};
    use serde::ser::{Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        creds: &Option<Credentials>,
        ser: S,
    ) -> Result<S::Ok, S::Error> {
        creds.clone().map(Shim::from).serialize(ser)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        de: D,
    ) -> Result<Option<Credentials>, D::Error> {
        let shim = Option::<Shim>::deserialize(de)?;
        Ok(shim.map(Into::into))
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
