extern crate repo_backup;
extern crate shellexpand;
extern crate failure;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use std::path::PathBuf;
use failure::{ResultExt, Error};
use structopt::StructOpt;
use repo_backup::Config;


fn main() {
    let args = Args::from_args();

    if let Err(e) = run(&args) {
        eprintln!("Error: {}", e);
        for cause in e.causes().skip(1) {
            eprintln!("\tCaused By: {}", cause);
        }

        eprintln!("{}", e.backtrace());
    }
}

fn run(args: &Args) -> Result<(), Error> {
    let cfg = args.config()?;

    println!("It Works!");

    Ok(())
}

#[derive(Debug, Clone, PartialEq, StructOpt)]
struct Args {
    #[structopt(short = "c", long = "config", 
                default_value = "~/.repo-backup.toml", help = "The configuration file to use.")]
    config_file: String,
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