extern crate env_logger;
extern crate github_backup;
extern crate dotenv;
extern crate log;
#[macro_use]
extern crate clap;

use std::env;
use std::process;
use std::io::{self, Write};
use clap::{Arg, ArgMatches};
use log::LogLevel;
use env_logger::LogBuilder;

use github_backup::errors::*;


macro_rules! backtrace {
    ($maybe_err:expr) => {
        match $maybe_err {
            Ok(val) => val,
            Err(e) => {
                print_backtrace(&e, 0);
                process::exit(1);
            }
        }
    }
}


fn main() {
    let (token, backup_dir) = parse_args();

    let repositories = backtrace!(github_backup::get_repos(&token));
    for repo in repositories {
        backtrace!(github_backup::backup_repo(&repo, &backup_dir));
    }
}


/// Try to find the GitHub API token.
///
/// This will first check the command line arguments, falling back to
/// the `GITHUB_TOKEN` environment variable. If no token is found,
/// log the error and exit.
fn token(matches: &ArgMatches) -> String {
    if let Some(tok) = matches.value_of("token") {
        return tok.to_string();
    } else {
        match env::var("GITHUB_TOKEN").ok() {
            Some(tok) => tok,
            None => {
                let stderr = io::stderr();
                writeln!(stderr.lock(), "No API token found.").ok();
                writeln!(stderr.lock(), 
                    "(Note: you can provide it using the GITHUB_TOKEN environment variable or the -t flag)").ok();
                process::exit(1);
            }
        }
    }
}

fn init_logger(verbose: u64) {
    let log_level = match verbose {
        0 => LogLevel::Warn,
        1 => LogLevel::Info,
        2 => LogLevel::Debug,
        _ => LogLevel::Trace,
    };

    LogBuilder::new()
        .filter(Some("github_backup"), log_level.to_log_level_filter())
        .init()
        .ok();
}

fn print_backtrace(e: &Error, indent: usize) {
    let stderr = io::stderr();
    writeln!(stderr.lock(), "{}Error: {}", "\t".repeat(indent), e).unwrap();

    for cause in e.iter().skip(1) {
        writeln!(
            stderr.lock(),
            "{}Caused By: {}",
            "\t".repeat(indent + 1),
            cause
        ).unwrap();
    }
}

fn parse_args() -> (String, String) {
    dotenv::dotenv().ok();

    let matches = app_from_crate!()
        .arg(
            Arg::with_name("token")
                .short("t")
                .long("token")
                .takes_value(true)
                .help(
                    "Your GitHub API token (defaults to GITHUB_TOKEN env variable)",
                ),
        )
        .arg(
            Arg::with_name("backup-dir")
                .short("d")
                .long("backup-dir")
                .help("The directory to save backups to.")
                .default_value("."),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Sets the verbosity level (repeat for more verbosity)"),
        )
        .get_matches();

    let verbose = matches.occurrences_of("verbose");
    init_logger(verbose);

    let tok = token(&matches);
    let backup_dir = matches.value_of("backup-dir").expect("unreachable");

    (tok, backup_dir.to_string())
}
