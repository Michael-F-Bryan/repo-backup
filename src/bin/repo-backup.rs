extern crate chrono;
extern crate env_logger;
extern crate failure;
#[macro_use]
extern crate log;
extern crate repo_backup;
extern crate shellexpand;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use std::env;
use std::io::{self, Write};

use chrono::Local;
use env_logger::Builder;
use failure::{Error, ResultExt};
use log::LevelFilter;
use repo_backup::{Config, Driver, UpdateFailure};
use structopt::StructOpt;

fn main() {
    let args = Args::from_args();

    if args.example_config {
        generate_example();
        return;
    }

    if let Err(e) = run(&args) {
        if let Some(outcome_failure) = e.downcast_ref::<UpdateFailure>() {
            let mut stderr = io::stderr();
            outcome_failure.display(&mut stderr).unwrap();
        } else {
            eprintln!("Error: {}", e);

            for cause in e.iter_chain().skip(1) {
                eprintln!("\tCaused By: {}", cause);
            }

            eprintln!("{}", e.backtrace());
        }
    }
}

fn generate_example() {
    let example = Config::example();

    println!("{}", example.as_toml());
}

fn run(args: &Args) -> Result<(), Error> {
    initialize_logging(args)?;
    let cfg = args.config()?;

    if log_enabled!(log::Level::Debug) {
        for line in format!("{:#?}", cfg).lines() {
            debug!("{}", line);
        }
    }

    let driver = Driver::with_config(cfg);

    driver.run()?;

    Ok(())
}

#[derive(Debug, Clone, PartialEq, StructOpt)]
struct Args {
    #[structopt(
        short = "c",
        long = "config",
        default_value = "~/.repo-backup.toml",
        help = "The configuration file to use."
    )]
    config_file: String,
    #[structopt(
        short = "v",
        long = "verbose",
        help = "Verbose output (repeat for more verbosity)",
        parse(from_occurrences)
    )]
    verbosity: u64,
    #[structopt(
        long = "example-config",
        help = "Generate an example config and immediately exit."
    )]
    example_config: bool,
}

impl Args {
    pub fn config(&self) -> Result<Config, Error> {
        let config_file = shellexpand::full(&self.config_file)
            .context("Unable to expand wildcards")?;

        Config::from_file(&*config_file)
            .context("Couldn't load the config")
            .map_err(Into::into)
    }
}

fn initialize_logging(args: &Args) -> Result<(), Error> {
    let mut builder = Builder::new();

    let level = match args.verbosity {
        0 => None,
        1 => Some(LevelFilter::Info),
        2 => Some(LevelFilter::Debug),
        _ => Some(LevelFilter::Trace),
    };

    if let Some(lvl) = level {
        builder.filter(Some("repo_backup"), lvl);
    }

    if let Ok(filter) = env::var("RUST_LOG") {
        builder.parse(&filter);
    }

    builder.format(|out, record| {
        match (record.line(), cfg!(debug_assertions)) {
            (Some(line), true) => writeln!(
                out,
                "{} [{:5}] ({}#{}): {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                line,
                record.args()
            ),
            _ => writeln!(
                out,
                "{} [{:5}]: {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            ),
        }
    });

    builder.try_init()?;

    Ok(())
}
