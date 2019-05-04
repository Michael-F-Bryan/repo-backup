#[macro_use]
extern crate actix;
use failure;


#[macro_use]
extern crate failure_derive;


#[macro_use]
extern crate slog;


#[macro_use]
extern crate serde_derive;



pub mod config;
mod driver;
mod git;
pub mod providers;

pub use crate::config::Config;
pub use crate::driver::{run, Driver};
pub use crate::git::GitRepo;
