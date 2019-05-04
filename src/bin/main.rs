extern crate repo_backup;
extern crate shellexpand;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;
extern crate structopt;

use slog::{Drain, Level, Logger};
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

fn main() {
    let args = Args::from_args();
    let logger = initialize_logging(&args);

    let code = repo_backup::run(args.config_file(), logger);
    process::exit(code);
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
    #[structopt(
        help = "The config file",
        default_value = "~/.repo-backup.toml"
    )]
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
