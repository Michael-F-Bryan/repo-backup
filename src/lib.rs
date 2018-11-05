#[macro_use]
extern crate actix;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate futures;
extern crate num_cpus;
#[macro_use]
extern crate slog;
extern crate serde;
extern crate toml;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate tempfile;

pub mod config;
mod driver;
mod git;
pub mod providers;

pub use crate::config::Config;
pub use crate::driver::Driver;
pub use crate::git::GitRepo;
