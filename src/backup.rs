use std::path::Path;
use std::fs;
use std::process::Command;

use raw_github::Repo;
use errors::*;


pub fn backup_repo<P: AsRef<Path>>(repo: &Repo, backup_dir: P) -> Result<()> {
    info!("Backing up {}", repo.full_name);

    let location = backup_dir.as_ref().join(&repo.full_name);

    if let Some(parent) = location.parent() {
        if !parent.exists() {
            fs::create_dir_all(&parent).chain_err(
                || "Couldn't create the target directory",
            )?;
        }
    }

    if !location.exists() {
        let cmd = format!("git clone --recurse-submodules {}", repo.clone_url);
        run_command(&location.parent().unwrap(), &cmd)?;
    } else {
        run_command(&location, "git pull --all")?;
        run_command(&location, "git submodule update --recursive --init")?;
    }

    Ok(())
}

fn run_command(dir: &Path, cmd: &str) -> Result<()> {
    let mut splits = cmd.split_whitespace();
    let name = splits.next().expect("Should always get something here");
    let args: Vec<&str> = splits.collect();

    trace!("Running command: {:?}", cmd);

    let output = Command::new(name)
        .args(&args)
        .current_dir(dir)
        .output()
        .chain_err(|| format!("Couldn't run subcommand: {:?}", cmd))?;

    trace!("Status: {:?}", output.status);
    trace!("Stdout: {:?}", String::from_utf8_lossy(&output.stdout));

    if output.status.success() {
        Ok(())
    } else {
        debug!("stderr for failed command: {:?}", String::from_utf8_lossy(&output.stderr));
        Err(ErrorKind::Subcommand(cmd.to_string(), output).into())
    }
}