use crate::git::GitRepo;
use failure::Error;
use futures::Stream;
mod github;

pub use self::github::{GitHub, GitHubConfig};

/// Something which can retrieve the repositories we want to backup.
pub trait Provider {
    fn repositories(&self) -> Box<Stream<Item = GitRepo, Error = Error>>;
}
