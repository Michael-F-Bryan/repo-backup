use serde::de::{
    Deserialize, DeserializeOwned, Deserializer, Error as DeError,
};
use serde::ser::{Error as SerError, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;
use toml::Value;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Config {
    pub general: General,
    pub rest: BTreeMap<String, Value>,
}

impl Config {
    pub fn from_toml(raw: &str) -> Result<Config, toml::de::Error> {
        toml::from_str(raw)
    }

    pub fn get_deserialized<D: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<D, ConfigError> {
        self.rest
            .get(key)
            .ok_or(ConfigError::MissingKey)
            .and_then(|v| v.clone().try_into().map_err(ConfigError::Toml))
    }
}

#[derive(Debug, Clone, Fail)]
pub enum ConfigError {
    MissingKey,
    Toml(toml::de::Error),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ConfigError::MissingKey => write!(f, "missing key"),
            ConfigError::Toml(ref t) => t.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct General {
    /// The top-level directory all backups should be placed in.
    pub root: PathBuf,
    pub threads: usize,
    /// The maximum number of errors allowed before declaring the entire backup
    /// as failed.
    ///
    /// A threshold of `0` means there's no limit.
    pub error_threshold: usize,
    pub blacklist: Vec<PathBuf>,
}

impl Default for General {
    fn default() -> General {
        General {
            root: PathBuf::from("."),
            threads: num_cpus::get(),
            error_threshold: 0,
            blacklist: Vec::new(),
        }
    }
}

impl Serialize for Config {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let mut merged = self.rest.clone();
        let general =
            Value::try_from(&self.general).map_err(S::Error::custom)?;
        merged.insert("general".into(), general);

        merged.serialize(ser)
    }
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let mut merged = BTreeMap::<String, Value>::deserialize(de)?;
        let general = match merged.remove("general") {
            Some(got) => got.try_into().map_err(D::Error::custom)?,
            None => Default::default(),
        };

        Ok(Config {
            general,
            rest: merged,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_the_config() {
        let cfg = Config {
            general: General {
                root: PathBuf::from("/path/to/backups"),
                threads: 42,
                error_threshold: 5,
                blacklist: Vec::new(),
            },
            rest: vec![(String::from("first"), Value::Integer(1))]
                .into_iter()
                .collect(),
        };

        let as_str = toml::to_string(&cfg).unwrap();
        let round_tripped: Config = toml::from_str(&as_str).unwrap();

        assert_eq!(round_tripped, cfg);
    }
}
