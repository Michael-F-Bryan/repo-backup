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