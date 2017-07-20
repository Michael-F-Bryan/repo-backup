use std::path::Path;
use std::fs;
use std::process::Command;

use raw_github::Repo;
use errors::*;


/// Backup a single repository in the provided `backup_dir`.
///
/// This uses the repo's `full_name` joined with `backup_dir` as the location
/// to clone into. If that already exists, it'll `cd` into that directory and
/// run `git pull`, as well as update any git submodules if applicable.
pub fn backup_repo<P: AsRef<Path>>(repo: &Repo, backup_dir: P) -> Result<()> {
    info!("Backing up {}", repo.full_name);

    let location = backup_dir.as_ref().join(&repo.full_name);

    if let Some(parent) = location.parent() {
        if !parent.exists() {
            fs::create_dir_all(&parent).chain_err(
                || format!("Couldn't create the target directory ({})", parent.display())
            )?;
        }
    }

    if !location.exists() {
        let cmd = format!("git clone --recurse-submodules {}", repo.clone_url);
        run_command(repo, location.parent().unwrap(), &cmd)?;
    } else {
        run_command(repo, &location, "git pull --all")?;
        run_command(repo, &location, "git submodule update --recursive --init")?;
    }

    info!("{} is up to date", repo.full_name);
    Ok(())
}

fn run_command(repo: &Repo, dir: &Path, cmd: &str) -> Result<()> {
    let mut splits = cmd.split_whitespace();
    let name = splits.next().expect("Should always get something here");
    let args: Vec<&str> = splits.collect();

    trace!("({}) Running command: {:?}", repo.full_name, cmd);

    let output = Command::new(name)
        .args(&args)
        .current_dir(dir)
        .output()
        .chain_err(|| format!("Couldn't run subcommand: {:?}", cmd))?;

    trace!("({}) Exit Status: {}", repo.full_name, output.status);
    if !output.stdout.is_empty() {
        trace!("({}) Stdout: {:?}", repo.full_name, String::from_utf8_lossy(&output.stdout));
    }

    if output.status.success() {
        Ok(())
    } else {
        debug!("({}) stderr for failed command: {:?}", repo.full_name, String::from_utf8_lossy(&output.stderr));
        Err(ErrorKind::Subcommand(repo.clone(), cmd.to_string(), output).into())
    }
}