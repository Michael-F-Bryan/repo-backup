use actix::{
    Actor, Addr, Arbiter, AsyncContext, Context, Handler, Recipient,
    SyncArbiter, System,
};
use crate::git::{DownloadRepo, GitClone};
use crate::providers::Provider;
use crate::Config;
use failure::Error;
use futures::future::{self, Future};
use futures::stream::{self, Stream};
use slog::Logger;

pub struct Driver {
    config: Config,
    logger: Logger,
    providers: Vec<Box<Provider>>,
    gits: Recipient<DownloadRepo>,
}

impl Driver {
    /// Create a new driver which will download repositories on a background
    /// thread pool.
    pub fn new(config: Config, logger: Logger) -> Driver {
        let l2 = logger.clone();
        let gits = SyncArbiter::start(config.general.threads, move || {
            GitClone::new(l2.clone())
        });

        Driver::new_with_recipient(config, logger, gits.recipient())
    }

    pub fn new_with_recipient(
        config: Config,
        logger: Logger,
        gits: Recipient<DownloadRepo>,
    ) -> Driver {
        Driver {
            config,
            logger,
            providers: Vec::new(),
            gits,
        }
    }

    pub fn register<P: Provider + 'static>(&mut self, provider: P) {
        self.providers.push(Box::new(provider));
    }

    pub fn do_register<F, P>(&mut self, constructor: F)
    where
        F: FnOnce(&Config, &Logger) -> P,
        P: Provider + 'static,
    {
        let provider = constructor(&self.config, &self.logger);
        self.register(provider);
    }
}

impl Actor for Driver {
    type Context = Context<Driver>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let repos = stream::iter_ok::<_, Error>(
            self.providers
                .iter()
                .map(|p| p.repositories())
                .collect::<Vec<_>>(),
        )
        .flatten();

        let gits = self.gits.clone();

        let started_downloading = repos
            .and_then(move |repo| gits.send(DownloadRepo(repo)).from_err());

        let this = ctx.address();
        let finished_downloading = started_downloading
            .and_then(move |result| this.send(Done(result)).from_err());

        let logger = self.logger.clone();
        let this = ctx.address();
        Arbiter::spawn(
            finished_downloading
                .for_each(|_| future::ok(()))
                .map_err(move |e| {
                    error!(logger, "Error!";
                "error" => e.to_string())
                })
                .then(move |_| this.send(Stop).map_err(|_| ())),
        );
    }
}

#[derive(Debug, Message)]
struct Stop;

impl Handler<Stop> for Driver {
    type Result = ();

    fn handle(&mut self, _msg: Stop, ctx: &mut Self::Context) {
        info!(self.logger, "Stopping...");
        System::current().stop();
    }
}

#[derive(Debug, Message)]
struct Done(Result<(), Error>);

impl Handler<Done> for Driver {
    type Result = ();

    fn handle(&mut self, msg: Done, _ctx: &mut Self::Context) {
        unimplemented!();
    }
}

/// Convert a future which returns a result into a future which will error when
/// the inner result errors.
fn lift_err<T, E>(
    fut: impl Future<Item = Result<T, impl Into<E>>, Error = impl Into<E>>,
) -> impl Future<Item = T, Error = E> {
    fut.map_err(Into::into)
        .then(|item| item.map(|inner| inner.map_err(Into::into)))
        .flatten()
}
