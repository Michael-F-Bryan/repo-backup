//! A small utility for making a local copy of all your GitHub projects.

extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

pub mod config;
mod driver;
mod github;
mod utils;

pub use config::Config;
pub use driver::Driver;
pub use github::GitHub;

use failure::{Error, SyncFailure};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Repo {
    owner: String,
    name: String,
    provider: String,
    url: String,
}

pub trait Provider {
    /// The `Provider`'s name.
    fn name(&self) -> &str;

    /// Get an iterator over all the available repositories.
    fn repositories(&self) -> Result<Vec<Repo>, Error>;
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
