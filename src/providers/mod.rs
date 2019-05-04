use crate::git::GitRepo;
use failure::Error;
use futures::Stream;
mod github;
mod gitlab;

pub use self::github::{GitHub, GitHubConfig};
pub use self::gitlab::{GitLab, GitLabConfig};

/// Something which can retrieve the repositories we want to backup.
pub trait Provider {
    fn repositories(&self) -> Box<dyn Stream<Item = GitRepo, Error = Error>>;
}
