use actix::{Actor, Addr, Handler, Message, SyncContext};
use crate::driver::Driver;
use failure::Error;
use slog::Logger;
use std::path::PathBuf;

#[derive(Debug, Clone, Message)]
pub(crate) struct GitClone {
    logger: Logger,
}

impl GitClone {
    pub fn new(logger: Logger) -> GitClone {
        GitClone { logger }
    }
}

impl Actor for GitClone {
    type Context = SyncContext<GitClone>;
}

impl Handler<DownloadRepo> for GitClone {
    type Result = Result<(), Error>;

    fn handle(
        &mut self,
        msg: DownloadRepo,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let DownloadRepo(repo) = msg;

        debug!(self.logger, "Started downloading a repository";
            "dest-dir" => repo.dest_dir.display(),
            "url" => &repo.ssh_url);

        Ok(())
    }
}

/// Request that a repository is downloaded.
#[derive(Clone, PartialEq)]
pub struct DownloadRepo(pub GitRepo);

impl Message for DownloadRepo {
    type Result = Result<(), Error>;
}

/// A basic git repository.
#[derive(Debug, Clone, PartialEq)]
pub struct GitRepo {
    pub dest_dir: PathBuf,
    pub ssh_url: String,
}
