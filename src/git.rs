use actix::{Actor, Handler, Message, SyncContext};
use failure::{Error, ResultExt};
use slog::Logger;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Message)]
pub(crate) struct GitClone {
    logger: Logger,
}

impl GitClone {
    pub fn new(logger: Logger) -> GitClone {
        GitClone { logger }
    }
}

impl Actor for GitClone {
    type Context = SyncContext<GitClone>;
}

impl Handler<DownloadRepo> for GitClone {
    type Result = Result<(), Error>;

    fn handle(
        &mut self,
        msg: DownloadRepo,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let DownloadRepo(GitRepo { ssh_url, dest_dir }) = msg;

        debug!(self.logger, "Started downloading a repository";
            "dest-dir" => dest_dir.display(),
            "url" => &ssh_url);

        if dest_dir.exists() {
            debug!(self.logger, "Fetching updates"; "ssh-url" => &ssh_url);
            fetch_updates(&dest_dir)
        } else {
            debug!(self.logger, "Cloning into repo"; "ssh-url" => &ssh_url);
            do_clone(&dest_dir, &ssh_url)
        }
    }
}

/// Request that a repository is downloaded.
#[derive(Debug, Clone, PartialEq)]
pub struct DownloadRepo(pub GitRepo);

impl Message for DownloadRepo {
    type Result = Result<(), Error>;
}

/// A basic git repository.
#[derive(Debug, Clone, PartialEq)]
pub struct GitRepo {
    pub dest_dir: PathBuf,
    pub ssh_url: String,
}

fn do_clone(dest_dir: &Path, ssh_url: &str) -> Result<(), Error> {
    let output = Command::new("git")
        .arg("clone")
        .arg("--quiet")
        .arg("--recursive")
        .arg(ssh_url)
        .arg(dest_dir)
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(
            failure::err_msg(String::from_utf8(output.stderr).unwrap_or_else(
                |_| String::from("<couldn't read the error message>"),
            ))
            .context("Unable to clone the repository")
            .into(),
        )
    }
}

fn fetch_updates(dest_dir: &Path) -> Result<(), Error> {
    can_update_git_repo(&dest_dir)?;

    let output = Command::new("git")
        .arg("fetch")
        .arg("--all")
        .arg("--quiet")
        .arg("--tags")
        .arg("--prune")
        .arg("--recurse-submodules=yes")
        .current_dir(dest_dir)
        .output()
        .context("Unable to invoke git")?;

    if !output.status.success() {
        let err =
            failure::err_msg(String::from_utf8(output.stderr).unwrap_or_else(
                |_| String::from("<couldn't read the error message>"),
            ))
            .context("Unable to fetch upstream changes");

        return Err(err.into());
    }

    let output = Command::new("git")
        .arg("merge")
        .arg("--ff-only")
        .arg("--quiet")
        .arg("FETCH_HEAD")
        .current_dir(dest_dir)
        .output()
        .context("Unable to invoke git")?;

    if !output.status.success() {
        let err =
            failure::err_msg(String::from_utf8(output.stderr).unwrap_or_else(
                |_| String::from("<couldn't read the error message>"),
            ))
            .context("Unable to fast-forward to the latest changes");

        return Err(err.into());
    }

    Ok(())
}

fn can_update_git_repo(repo_dir: &Path) -> Result<(), Error> {
    if !repo_dir.join(".git").is_dir() {
        return Err(NotARepo.into());
    }

    let git_status = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(repo_dir)
        .output()
        .context("Unable to invoke git")?;

    if !git_status.status.success() {
        let err = failure::err_msg(
            String::from_utf8(git_status.stderr).unwrap_or_else(|_| {
                String::from("<couldn't read the error message>")
            }),
        )
        .context("Unable to check if there are unsaved changes");

        return Err(err.into());
    }

    let stdout = String::from_utf8(git_status.stdout)
        .context("Can't parse output from `git status`")?;
    let lines = stdout.lines().count();

    if lines > 0 {
        return Err(UnsavedChanges { count: lines }.into());
    }

    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq, Fail)]
#[fail(display = "Not a git repository")]
struct NotARepo;

#[derive(Debug, Clone, PartialEq, Fail)]
struct UnsavedChanges {
    count: usize,
}

impl Display for UnsavedChanges {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "There are {} unsaved changes", self.count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::process::Stdio;

    macro_rules! require_program {
        ($name:expr) => {{
            let exists = ::std::process::Command::new($name)
                .arg("--help")
                .stdout(::std::process::Stdio::null())
                .stderr(::std::process::Stdio::null())
                .status()
                .is_ok();
            if !exists {
                eprintln!("Couldn't find \"{}\"", $name);
                return;
            }
        }};
    }

    #[test]
    fn directory_isnt_a_git_repo() {
        let temp = tempfile::tempdir().unwrap();

        let err = can_update_git_repo(temp.path()).unwrap_err();

        assert!(err.downcast_ref::<NotARepo>().is_some());
    }

    #[test]
    fn git_directory_with_unsaved_changes() {
        require_program!("git");

        let temp = tempfile::tempdir().unwrap();
        let status = Command::new("git")
            .arg("init")
            .arg(temp.path())
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success());
        File::create(temp.path().join("blah.txt")).unwrap();
        File::create(temp.path().join("second.txt")).unwrap();

        let err = can_update_git_repo(temp.path()).unwrap_err();

        let unsaved = err.downcast_ref::<UnsavedChanges>().unwrap();
        assert_eq!(unsaved.count, 2);
    }

    #[test]
    fn happy_git_directory() {
        require_program!("git");

        let temp = tempfile::tempdir().unwrap();
        let status = Command::new("git")
            .arg("init")
            .arg(temp.path())
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success());

        assert!(can_update_git_repo(temp.path()).is_ok());
    }

    #[test]
    fn clone_a_repo() {
        require_program!("git");

        let temp = tempfile::tempdir().unwrap();
        let sub_dir = temp.path().join("dest");

        do_clone(&sub_dir, env!("CARGO_MANIFEST_DIR")).unwrap();

        assert!(sub_dir.join(".git").exists());
    }

    #[test]
    fn clone_and_then_update() {
        require_program!("git");

        let temp = tempfile::tempdir().unwrap();
        let sub_dir = temp.path().join("dest");
        do_clone(&sub_dir, env!("CARGO_MANIFEST_DIR")).unwrap();

        assert!(fetch_updates(&sub_dir).is_ok());
    }
}
