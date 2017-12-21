//! A small utility for making a local copy of all your projects from a variety
//! of various sources.
//!
//! Sources currently supported:
//!
//! - [GitHub](https://github.com/)
//!
//!
//! # Configuration
//!
//! Configuration is done via a `repo-backup.toml` file. By default the
//! `repo-backup` program will look for this in your home directory (as
//! `~/.repo-backup.toml`), but this can be overridden via the command line.
//!
//! The configuration file looks something like this:
//!
//! ```rust
//! # use repo_backup::Config;
//! # let src = r#"
//! [general]
//! dest-dir = "/srv"
//!
//! [github]
//! api-key = "your API key"
//! owned = true
//! starred = false
//!
//! [gitlab]
//! api-key = "your API key"
//! url = "https://gitlab.com/"
//! organisations = true
//! owned = true
//! # "#;
//! # let example = Config::from_str(src).unwrap();
//! # assert_eq!(example, Config::example());
//! ```
//!
//! The only required table is `general`, with the others used to enable and
//! configure the corresponding [`Provider`].
//!
//! # Examples
//!
//! This crate is designed to be really easy to use as both an executable, *and*
//! a library.
//!
//! The [`Driver`] will:
//!
//! - Query each [`Provider`] in the [`Config`] for the available repositories, and
//! - Download each repository to [`dest_dir`].
//!
//! ```rust,no_run
//! # extern crate repo_backup;
//! # extern crate failure;
//! # use failure::Error;
//! use repo_backup::{Config, Driver};
//!
//! # fn run() -> Result<(), Error> {
//! let cfg = Config::from_file("/path/to/repo-backup.toml")?;
//! let driver = Driver::with_config(cfg);
//!
//! driver.run()?;
//! # Ok(())
//! # }
//! # fn main() { run().unwrap() }
//! ```
//!
//! Or if you want control over the list fetching and download process (e.g.
//! to add a couple extra git repos to the list or use your own [`Provider`]):
//!
//! ```rust,no_run
//! # extern crate repo_backup;
//! # extern crate failure;
//! # use failure::Error;
//! use repo_backup::{Config, Driver, Repo, Provider};
//!
//! struct MyCustomProvider;
//!
//! impl Provider for MyCustomProvider {
//!     fn name(&self) -> &str {
//!         "custom-provider"
//!     }
//!
//!     fn repositories(&self) -> Result<Vec<Repo>,  Error> {
//!         unimplemented!()
//!     }
//! }
//!
//! # fn run() -> Result<(), Error> {
//! let cfg = Config::from_file("/path/to/repo-backup.toml")?;
//! let driver = Driver::with_config(cfg);
//!
//! let providers: Vec<Box<Provider>> = vec![Box::new(MyCustomProvider)];
//! let mut repos = driver.get_repos_from_providers(&providers)?;
//!
//! let my_repo = Repo {
//!     name: String::from("My Repo"),
//!     owner: String::from("Michael-F-Bryan"),
//!     provider: String::from("custom"),
//!     url: String::from("http://my.git.server/Michael-F-Bryan/my_repo"),
//! };
//! repos.push(my_repo);
//!
//! driver.update_repos(&repos)?;
//! # Ok(())
//! # }
//! # fn main() { run().unwrap() }
//! ```
//!
//! [`Driver`]: struct.Driver.html
//! [`Provider`]: trait.Provider.html
//! [`Config`]: config/struct.Config.html
//! [`dest_dir`]: config/struct.General.html#structfield.dest_dir

#![deny(missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
        trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
        unused_imports, unused_qualifications)]

extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate gitlab;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate sec;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

#[macro_use]
mod utils;
pub mod config;
mod driver;
mod github;
mod gitlab_provider;

pub use config::Config;
pub use driver::{Driver, UpdateFailure};
pub use github::GitHub;
pub use gitlab_provider::Gitlab;

use failure::{Error, SyncFailure};

/// A repository.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Repo {
    /// The repository's owner.
    pub owner: String,
    /// The name of the repository.
    pub name: String,
    /// Which provider this repository was retrieved from.
    pub provider: String,
    /// A URL which can be used when downloading the repo.
    pub url: String,
}

impl Repo {
    /// Get the repository's canonical name in `$provider/$owner/$name` form
    /// (e.g. `github/Michael-F-Bryan/repo-backup`).
    pub fn full_name(&self) -> String {
        format!("{}/{}/{}", self.provider, self.owner, self.name)
    }
}

/// A source of repositories.
pub trait Provider {
    /// The `Provider`'s name.
    fn name(&self) -> &str;

    /// Get a list of all the available repositories from this source.
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
