//! A small utility for making a local copy of all your GitHub projects.

#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate github_rs;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

pub mod config;
mod driver;
mod github;

pub use config::Config;
pub use driver::Driver;
pub use github::GitHub;

use std::path::Path;
use failure::{Error, SyncFailure};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Repo {
    owner: String,
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

trait SyncResult<T, E> {
    fn sync(self) -> Result<T, SyncFailure<E>>
    where
        Self: Sized,
        E: ::std::error::Error + Send + 'static;
}

impl<T, E> SyncResult<T, E> for Result<T, E> {
    fn sync(self) -> Result<T, SyncFailure<E>>
    where
        Self: Sized,
        E: ::std::error::Error + Send + 'static,
    {
        self.map_err(SyncFailure::new)
    }
}
