extern crate env_logger;
extern crate github_backup;
extern crate dotenv;
#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use std::env;
use std::process;
use std::io::{self, Write};
use clap::{Arg, ArgMatches};
use log::LogLevel;
use env_logger::LogBuilder;

use github_backup::Client;
use github_backup::errors::*;



macro_rules! backtrace {
    ($maybe_err:expr) => {
        match $maybe_err {
            Ok(val) => val,
            Err(e) => {
                println!("{:#?}", e);
                print_backtrace(&e, 0);
                process::exit(1);
            }
        }
    }
}


fn main() {
    dotenv::dotenv().ok();

    let matches = app_from_crate!()
        .arg(Arg::with_name("token").short("t").long("token").help(
            "Your GitHub API token (defaults to GITHUB_TOKEN env variable)",
        ))
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

    let token = get_token(&matches);
    let backup_dir = matches.value_of("backup-dir").expect("unreachable");

    let client = backtrace!(Client::new(token, backup_dir));

    backtrace!(client.run());
}


/// Try to find the GitHub API token.
///
/// This will first check the command line arguments, falling back to
/// the `GITHUB_TOKEN` environment variable. If no token is found,
/// log the error and exit.
fn get_token(matches: &ArgMatches) -> String {
    if let Some(tok) = matches.value_of("token") {
        return tok.to_string();
    } else {
        match env::var("GITHUB_TOKEN").ok() {
            Some(tok) => tok,
            None => {
                error!("No token found");
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

        if let ErrorKind::FailedUpdate(ref name, ref errs) = *e.kind() {
            println!("{} {:?}", name, errs);
            for err in errs {
                print_backtrace(err, indent + 2);
            }
        }
    }
}