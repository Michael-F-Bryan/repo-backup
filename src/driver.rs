use actix::{
    Actor, Arbiter, AsyncContext, Context, Handler, Recipient, SyncArbiter,
    System,
};
use crate::config::{Config, ConfigError};
use crate::git::{DownloadRepo, GitClone, GitRepo};
use crate::providers::{GitHub, GitLab, Provider};
use failure::Error;
use futures::future::{self, Future};
use futures::stream::{self, Stream};
use serde::Deserialize;
use slog::Logger;
use std::fs;
use std::path::Path;

macro_rules! try {
    ($result:expr, $logger:expr) => {
        try!($result, $logger, "Oops...");
    };
    ($result:expr, $logger:expr, $err_msg:expr) => {
        match $result {
            Ok(r) => r,
            Err(e) => {
                let err_msg = $err_msg;
                let logger = $logger;
                error!(logger, "{}", err_msg; "error" => e.to_string());

                return 1;
            }
        }
    };
}

pub fn run<P: AsRef<Path>>(config: P, logger: Logger) -> i32 {
    let config = config.as_ref();

    let cfg = try!(
        fs::read_to_string(&config)
            .map_err(Error::from)
            .and_then(|s| Config::from_toml(&s).map_err(Error::from)),
        &logger,
        "Unable to load the config"
    );

    let sys = System::new("repo-backup");

    let mut driver = Driver::new(cfg.clone(), logger.clone());
    register_providers(&mut driver, &cfg, &logger);
    driver.start();

    info!(logger, "Started the backup process"; 
        "config-file" => config.display(),
        "root" => cfg.general.root.display(),
        "threads" => cfg.general.threads,
        "error-threshold" => cfg.general.error_threshold);
    sys.run()
}

fn register_providers(driver: &mut Driver, cfg: &Config, logger: &Logger) {
    debug!(logger, "Registering providers");

    try_register("github", &cfg, driver, logger, |got, logger| {
        debug!(logger, "Registering the GitHub provider");
        GitHub::new(got, logger.clone())
    });
    try_register("gitlab", &cfg, driver, logger, |got, logger| {
        debug!(logger, "Registering the GitLab provider");
        GitLab::new(got, logger.clone())
    });
}

/// Try to parse the corresponding section from a `Config`, if successful use
/// the resulting value to construct a `Provider` to be registered with the
/// `Driver`.
fn try_register<F, P, C>(
    key: &str,
    cfg: &Config,
    driver: &mut Driver,
    logger: &Logger,
    then: F,
) where
    F: FnOnce(C, &Logger) -> P,
    P: Provider + 'static,
    C: for<'de> Deserialize<'de>,
{
    match cfg.get_deserialized(key) {
        Ok(got) => {
            let provider = then(got, logger);
            driver.register(provider);
        }
        Err(ConfigError::Toml(toml)) => {
            warn!(logger, "Unable to parse the \"{}\" config section", key;
                "error" => toml.to_string());
        }
        Err(ConfigError::MissingKey) => {}
    }
}

pub struct Driver {
    config: Config,
    logger: Logger,
    providers: Vec<Box<Provider>>,
    gits: Recipient<DownloadRepo>,
    stats: Statistics,
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
            stats: Statistics::default(),
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
        let blacklist = self.config.general.blacklist.clone();
        let logger = self.logger.clone();

        let started_downloading = repos
            .filter(move |repo| {
                let ignore = blacklist
                    .iter()
                    .any(|item| Path::new(&item) == &repo.dest_dir);

                if ignore {
                    info!(logger, "Ignoring repo";
                            "dest-dir" => repo.dest_dir.display());
                }
                !ignore
            }).and_then(move |repo| {
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
                .map_err(
                    move |e| error!(logger, "Error!"; "error" => e.to_string()),
                ).then(move |_| this.send(Stop).map_err(|_| ())),
        );
    }
}

#[derive(Debug, Message)]
struct Stop;

impl Handler<Stop> for Driver {
    type Result = ();

    fn handle(&mut self, _msg: Stop, _ctx: &mut Self::Context) {
        info!(self.logger, "Stopping...";
            "error-count" => self.stats.error_count,
            "successful-updates" => self.stats.success);
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
                "dest" => msg.repo.dest_dir.display(),
                "url" => &msg.repo.ssh_url);

            for cause in e.iter_causes() {
                warn!(self.logger, "Caused By"; "cause" => cause.to_string());
            }

            self.stats.error_count += 1;
            let threshold = self.config.general.error_threshold;

            if threshold > 0 && self.stats.error_count >= threshold {
                error!(self.logger, "Too many errors were encountered. Bailing";
                    "error-count" => self.stats.error_count);

                System::current().stop_with_code(1);
            }
        } else {
            info!(self.logger, "Successfully backed up a repo";
                "repo" => msg.repo.dest_dir.display());
            self.stats.success += 1;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Statistics {
    error_count: usize,
    success: usize,
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

        let code = sys.run();
        assert_eq!(code, 1);
    }
}
