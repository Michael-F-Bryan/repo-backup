use std::path::{Path, PathBuf};
use std::io::Read;
use std::fs::File;

use failure::{Error, ResultExt, Fail};
use toml;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    general: General,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct General {
    destination_dir: PathBuf,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Config, Error> {
        let file = file.as_ref();

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
