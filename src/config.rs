use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub general: General,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct General {
    /// The top-level directory all backups should be placed in.
    pub root: PathBuf,
    #[serde(default = "num_cpus::get")]
    pub threads: usize,
    /// The maximum number of errors allowed before declaring the entire backup
    /// as failed.
    ///
    /// A threshold of `0` means there's no limit.
    #[serde(default = "Default::default")]
    pub error_threshold: usize,
}
