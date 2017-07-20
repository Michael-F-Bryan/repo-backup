//! A small utility for making a local copy of all your GitHub projects.
//!
//! The main component of this project is a small command-line utility
//! called `github-backup`. It's designed to be quiet by default because when
//! run as a `cron` job, cron will mail you any time a job prints to
//! stdout/stderr or if it fails. If you want verbose output, use the `-v`
//! flag. The more times you pass in `-v`, the more verbose the output.
//!
//! # Examples
//!
//! ```rust,no_run
//! # fn run() -> ::github_backup::errors::Result<()> {
//! const MY_TOKEN: &'static str = "SECRET API TOKEN";
//! let backup_dir = ".";
//! let repos = github_backup::get_repos(MY_TOKEN)?;
//!
//! for repo in repos {
//!   github_backup::backup_repo(&repo, backup_dir)?;
//! }
//! # Ok(())
//! # }
//! # fn main() { run().unwrap() }
//! ```

#![deny(missing_docs)]

extern crate hyper;
extern crate dotenv;
extern crate github_rs;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

#[cfg(test)]
extern crate tempdir;

pub mod errors;
mod client;
mod raw_github;
mod backup;

pub use client::get_repos;
pub use backup::backup_repo;
pub use raw_github::Repo;