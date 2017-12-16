use std::path::{Path, PathBuf};
use std::io::Read;
use std::fs::File;

use failure::{Error, ResultExt};
use toml;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub general: General,
    pub github: Option<GithubConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct General {
    pub dest_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GithubConfig {
    pub api_key: String,
    #[serde(default = "always_true")] pub starred: bool,
    #[serde(default = "always_true")] pub owned: bool,
}

fn always_true() -> bool {
    true
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Config, Error> {
        let file = file.as_ref();
        debug!("Reading config from {}", file.display());

        let mut buffer = String::new();
        File::open(file)
            .with_context(|_| format!("Unable to open {}", file.display()))?
            .read_to_string(&mut buffer)
            .context("Reading config file failed")?;

        toml::from_str(&buffer)
            .context("Parsing config file failed")
            .map_err(Error::from)
    }
}
