use std::path::PathBuf;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    general: General,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct General {
    destination_dir: PathBuf,
}
