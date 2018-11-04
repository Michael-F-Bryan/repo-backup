//! Configuration for `repo-backup`.

use sec::Secret;
use serde::de::{Deserialize, Deserializer};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use failure::{Error, ResultExt};
use toml;

/// The overall configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// General configuration options.
    pub general: General,
    /// Settings specific to the `Github` provider.
    pub github: Option<GithubConfig>,
    /// Settings for the `GitLab` provider.
    pub gitlab: Option<GitLabConfig>,
}

/// General settings used by `repo-backup`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct General {
    /// The root directory to place all downloaded repositories.
    pub dest_dir: PathBuf,
    /// The maximum number of errors that can be encountered before bailing.
    #[serde(default, deserialize_with = "deserialize_error_threshold")]
    pub max_error_threshold: Option<usize>,
}

fn deserialize_error_threshold<'de, D: Deserializer<'de>>(
    de: D,
) -> Result<Option<usize>, D::Error> {
    let raw: Option<usize> = Deserialize::deserialize(de)?;
    if raw == Some(0) {
        Ok(None)
    } else {
        Ok(raw)
    }
}

/// Github-specific settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GithubConfig {
    /// The API key to use. You will need to [create a new personal access
    /// token][new] and give it the `public_repo` permissions before you can
    /// fetch repos from GitHub.
    ///
    /// [new]: https://github.com/settings/tokens/new
    pub api_key: Secret<String>,
    /// Should we download all starred repos? (default: true)
    #[serde(default = "always_true")]
    pub starred: bool,
    /// Should we download all owned repos? (default: true)
    #[serde(default = "always_true")]
    pub owned: bool,
}

/// Github-specific settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[allow(deprecated)]
pub struct GitLabConfig {
    /// The API key to use. Make sure you create a new [personal access token][new]
    /// and give it the "api" scope, if you haven't already.
    ///
    /// [new]: https://gitlab.com/profile/personal_access_tokens
    pub api_key: Secret<String>,
    /// Hostname of the GitLab instance to fetch repositories from.
    #[serde(default = "default_gitlab_url")]
    pub host: String,
    /// Should we download all repos owned by organisations you are a part of?
    /// (default: false)
    #[serde(default = "always_false")]
    pub organisations: bool,
    /// Should we download all owned repos? (default: true)
    #[serde(default = "always_true")]
    pub owned: bool,
}

fn always_true() -> bool {
    true
}

fn always_false() -> bool {
    false
}

fn default_gitlab_url() -> String {
    String::from("https://gitlab.com/")
}

impl Config {
    /// Load a `Config` from some file on disk.
    pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Config, Error> {
        let file = file.as_ref();
        debug!("Reading config from {}", file.display());

        let mut buffer = String::new();
        File::open(file)
            .with_context(|_| format!("Unable to open {}", file.display()))?
            .read_to_string(&mut buffer)
            .context("Reading config file failed")?;

        Config::from_str(&buffer)
    }

    /// Load the config directly from a source string.
    pub fn from_str(src: &str) -> Result<Config, Error> {
        toml::from_str(src)
            .context("Parsing config file failed")
            .map_err(Error::from)
    }

    /// Generate an example config.
    pub fn example() -> Config {
        Config {
            general: General {
                dest_dir: PathBuf::from("/srv"),
                max_error_threshold: None,
            },
            github: Some(GithubConfig {
                api_key: String::from("your API key").into(),
                owned: true,
                starred: false,
            }),
            gitlab: Some(GitLabConfig {
                api_key: String::from("your API key").into(),
                host: String::from("gitlab.com"),
                organisations: true,
                owned: true,
            }),
        }
    }

    /// Serialize the `Config` as TOML.
    pub fn as_toml(&self) -> String {
        match toml::to_string_pretty(self) {
            Ok(s) => s,
            Err(e) => {
                panic!("Serializing a Config should never fail. {}", e);
            }
        }
    }
}
