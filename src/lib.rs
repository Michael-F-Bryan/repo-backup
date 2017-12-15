//! A small utility for making a local copy of all your GitHub projects.

#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

pub mod config;
mod driver;
mod github;

pub use config::Config;
pub use driver::Driver;
pub use github::GitHub;

use std::path::Path;
use failure::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Repo {
    user: String,
    name: String,
    url: String,
}

pub trait Provider {
    /// The `Provider`'s name.
    fn name(&self) -> &str;

    /// Get an iterator over all the available repositories.
    fn repositories(&self) -> Result<Vec<Repo>, Error>;

    /// Download a specific repo.
    fn download(&self, repo: &Repo, destination: &Path) -> Result<(), Error>;
}
