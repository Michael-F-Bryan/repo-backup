use crate::git::GitRepo;
use failure::Error;
use futures::Stream;

pub trait Provider {
    fn repositories(&self) -> Box<Stream<Item = GitRepo, Error = Error>>;
}
