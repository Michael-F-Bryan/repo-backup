use actix::msgs::StopArbiter;
use actix::{
    Actor, Arbiter, AsyncContext, Context, Handler, Recipient, SyncArbiter,
    System,
};
use crate::git::{DownloadRepo, GitClone, GitRepo};
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
        let root = config.general.root.clone();
        let gits = SyncArbiter::start(config.general.threads, move || {
            GitClone::new(root.clone(), l2.clone())
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
        ).flatten();

        let gits = self.gits.clone();

        let started_downloading = repos.and_then(move |repo| {
            (
                future::ok(repo.clone()),
                gits.send(DownloadRepo(repo)).from_err(),
            )
        });

        let this = ctx.address();
        let finished_downloading =
            started_downloading.and_then(move |(repo, outcome)| {
                this.send(Done { repo, outcome }).from_err()
            });

        let logger = self.logger.clone();
        let this = ctx.address();
        Arbiter::spawn(
            finished_downloading
                .for_each(|_| future::ok(()))
                .map_err(move |e| {
                    error!(logger, "Error!";
                "error" => e.to_string())
                }).then(move |_| this.send(Stop).map_err(|_| ())),
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
struct Done {
    pub repo: GitRepo,
    pub outcome: Result<(), Error>,
}

impl Handler<Done> for Driver {
    type Result = ();

    fn handle(&mut self, msg: Done, _ctx: &mut Self::Context) {
        if let Err(e) = msg.outcome {
            warn!(self.logger, "Error backing up a repository";
                "error" => e.to_string(),
                "url" => &msg.repo.ssh_url);

            self.error_count += 1;
            let threshold = self.config.general.error_threshold;

            if threshold > 0 && self.error_count >= threshold {
                error!(self.logger, "Too many errors were encountered. Bailing";
                    "error-count" => self.error_count);

                System::current().arbiter().do_send(StopArbiter(1));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    struct DodgyActor;

    impl Actor for DodgyActor {
        type Context = Context<DodgyActor>;
    }

    impl Handler<DownloadRepo> for DodgyActor {
        type Result = Result<(), Error>;

        fn handle(
            &mut self,
            _msg: DownloadRepo,
            _ctx: &mut Self::Context,
        ) -> Self::Result {
            Err(failure::err_msg("Oops.."))
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
        let cfg = Config::default();
        let logger = Logger::root(Discard, o!());

        let sys = System::new("test");
        let mock = Mock {
            repos: Arc::clone(&repos),
        }.start();
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

    #[test]
    fn stop_after_encountering_the_error_threshold() {
        let mut cfg = Config::default();
        cfg.general.error_threshold = 1;

        let sys = System::new("test");
        let mut driver = Driver::new_with_recipient(
            cfg,
            Logger::root(Discard, o!()),
            DodgyActor.start().recipient(),
        );
        driver.register(MockProvider {
            repos: vec![
                GitRepo {
                    dest_dir: PathBuf::from("/1"),
                    ssh_url: String::from("1"),
                },
                GitRepo {
                    dest_dir: PathBuf::from("/1"),
                    ssh_url: String::from("1"),
                },
            ],
        });
        driver.start();

        let _code = sys.run();

        // FIXME: Figure out how to stop the system with an error code
        // assert_eq!(code, 1);
    }
}
