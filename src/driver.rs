use actix::{
    Actor, Arbiter, AsyncContext, Context, Handler, Recipient, SyncArbiter,
    System,
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
    error_count: usize,
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
            error_count: 0,
        }
    }

    pub fn register<P: Provider + 'static>(
        &mut self,
        provider: P,
    ) -> &mut Self {
        self.providers.push(Box::new(provider));
        self
    }

    pub fn do_register<F, P>(&mut self, constructor: F) -> &mut Self
    where
        F: FnOnce(&Config, &Logger) -> P,
        P: Provider + 'static,
    {
        let provider = constructor(&self.config, &self.logger);
        self.register(provider);
        self
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

    fn handle(&mut self, _msg: Stop, _ctx: &mut Self::Context) {
        info!(self.logger, "Stopping...");
        System::current().stop();
    }
}

#[derive(Debug, Message)]
struct Done(Result<(), Error>);

impl Handler<Done> for Driver {
    type Result = ();

    fn handle(&mut self, msg: Done, _ctx: &mut Self::Context) {
        if let Err(e) = msg.0 {
            warn!(self.logger, "Error backing up a repository";
                "error" => e.to_string());

            self.error_count += 1;
            let threshold = self.config.general.error_threshold;

            if threshold > 0 && self.error_count > threshold {
                error!(self.logger, "Bailing due to too many errors";
                    "error-count" => self.error_count);
                System::current().stop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::General;
    use crate::git::GitRepo;
    use slog::Discard;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    #[derive(Default, Debug, Clone)]
    struct Mock {
        repos: Arc<Mutex<Vec<DownloadRepo>>>,
    }

    impl Actor for Mock {
        type Context = Context<Mock>;
    }

    impl Handler<DownloadRepo> for Mock {
        type Result = Result<(), Error>;

        fn handle(
            &mut self,
            msg: DownloadRepo,
            _ctx: &mut Self::Context,
        ) -> Self::Result {
            self.repos.lock().unwrap().push(msg);
            Ok(())
        }
    }

    struct MockProvider {
        repos: Vec<GitRepo>,
    }

    impl Provider for MockProvider {
        fn repositories(&self) -> Box<Stream<Item = GitRepo, Error = Error>> {
            Box::new(stream::iter_ok(self.repos.clone()))
        }
    }

    fn dummy_config() -> Config {
        Config {
            general: General {
                root: PathBuf::new(),
                threads: 5,
                error_threshold: 0,
            },
        }
    }

    #[test]
    fn run_driver_to_completion() {
        let should_be = vec![
            GitRepo {
                dest_dir: PathBuf::from("/1"),
                ssh_url: String::from("1"),
            },
            GitRepo {
                dest_dir: PathBuf::from("/2"),
                ssh_url: String::from("2"),
            },
        ];

        let repos: Arc<Mutex<Vec<DownloadRepo>>> = Default::default();
        let cfg = dummy_config();
        let logger = Logger::root(Discard, o!());

        let sys = System::new("test");
        let mock = Mock {
            repos: Arc::clone(&repos),
        }
        .start();
        let mut driver =
            Driver::new_with_recipient(cfg, logger, mock.recipient());
        driver.register(MockProvider {
            repos: should_be.clone(),
        });
        driver.start();

        assert_eq!(sys.run(), 0);

        let got = repos
            .lock()
            .unwrap()
            .iter()
            .map(|repo| repo.0.clone())
            .collect::<Vec<_>>();
        assert_eq!(got, should_be);
    }
}
