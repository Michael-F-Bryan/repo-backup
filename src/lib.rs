//! A small utility for making a local copy of all your GitHub projects.

extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

pub mod config;

pub use config::Config;

use std::path::Path;
use failure::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Repo {
    user: String,
    name: String,
    url: String,
}

pub trait Provider {
    type Repositories: Iterator<Item = Repo>;

    /// The `Provider`'s name.
    fn name(&self) -> &str;

    /// Get an iterator over all the available repositories.
    fn repositories(&self) -> Result<Self::Repositories, Error>;

    /// Download a specific repo.
    fn download(&self, repo: &Repo, destination: &Path) -> Result<(), Error>;
}
