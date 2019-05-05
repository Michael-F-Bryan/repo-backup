use repo_backup;
use shellexpand;
#[macro_use]
extern crate slog;
use slog_async;
use slog_term;
use structopt;

use slog::{Drain, Level, Logger};
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

fn main() {
    let args = Args::from_args();
    let logger = initialize_logging(&args);

    if let Err(e) = repo_backup::run(args.config_file(), &logger) {
        error!(logger, "Error: {}", e);
        for cause in e.iter_causes() {
            warn!(logger, "Caused By: {}", cause);
        }

        drop(logger);

        let backtrace = e.backtrace().to_string();

        if !backtrace.trim().is_empty() {
            eprintln!("{}", backtrace);
        }

        process::exit(1);
    }
}

#[derive(Debug, Clone, StructOpt)]
pub struct Args {
    #[structopt(
        short = "v",
        long = "verbose",
        parse(from_occurrences),
        help = "Generate verbose output"
    )]
    verbosity: usize,
    #[structopt(help = "The config file", default_value = "~/.repo-backup.toml")]
    config: String,
}

impl Args {
    fn config_file(&self) -> PathBuf {
        shellexpand::full(&self.config)
            .map(|p| PathBuf::from(p.into_owned()))
            .unwrap_or_else(|_| PathBuf::from(&self.config))
    }
}

fn initialize_logging(args: &Args) -> Logger {
    let level = match args.verbosity {
        0 => Level::Warning,
        1 => Level::Info,
        2 => Level::Debug,
        _ => Level::Trace,
    };

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    Logger::root(drain.filter_level(level).fuse(), o!())
}
