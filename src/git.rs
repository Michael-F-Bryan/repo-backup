use actix::{Actor, Handler, Message, SyncContext};
use failure::{Error, ResultExt};
use slog::Logger;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Message)]
pub(crate) struct GitClone {
    logger: Logger,
    root: PathBuf,
}

impl GitClone {
    pub fn new(root: PathBuf, logger: Logger) -> GitClone {
        GitClone { root, logger }
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

        // make sure the path is absolute
        let dest_dir = self.root.join(dest_dir);

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
    /// The destination directory, relative to the backup root.
    pub dest_dir: PathBuf,
    pub ssh_url: String,
}

impl From<hubcaps::repositories::Repo> for GitRepo {
    fn from(other: hubcaps::repositories::Repo) -> GitRepo {
        GitRepo {
            dest_dir: PathBuf::from(other.full_name),
            ssh_url: other.ssh_url,
        }
    }
}

macro_rules! cmd {
    ($name:expr $(, $arg:expr)*) => {{
        let mut cmd = cmd!(@compose_cmd; $name $(, $arg)*);
        cmd!(@execute; cmd)
    }};
    ($name:expr $(, $arg:expr)*; in $current_dir:expr) => {{
        let mut cmd = cmd!(@compose_cmd; $name $(, $arg)*);
        cmd.current_dir($current_dir);
        cmd!(@execute; cmd)
    }};

    (@compose_cmd; $name:expr $(, $arg:expr)*) => {{
        let mut cmd = Command::new($name);
        $(
            cmd.arg($arg);
        )*
        cmd
    }};
    (@execute; $command:expr) => {{
        $command.output()
            .context("Unable to execute the command")
            .map_err(Error::from)
            .and_then(|output| if output.status.success() {
                Ok(output)
            } else {
                let stderr = String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| String::from("<couldn't read the error message>"));
                Err(failure::Error::from(failure::err_msg(stderr)))
            })
    }};
}

fn do_clone(dest_dir: &Path, ssh_url: &str) -> Result<(), Error> {
    cmd!("git", "clone", "--quiet", "--recursive", ssh_url, dest_dir)
        .context("Unable to clone the repository")?;

    Ok(())
}

fn fetch_updates(dest_dir: &Path) -> Result<(), Error> {
    can_update_git_repo(&dest_dir)?;

    cmd!("git", "fetch", "--all", "--quiet", "--tags", "--prune", 
        "--recurse-submodules=yes"; in dest_dir)
        .context("Unable to fetch upstream changes")?;

    cmd!("git", "merge", "--ff-only", "--quiet", "FETCH_HEAD"; in dest_dir)
        .context("Unable to fast-forward to the latest changes")?;

    Ok(())
}

fn can_update_git_repo(repo_dir: &Path) -> Result<(), Error> {
    if !repo_dir.join(".git").is_dir() {
        return Err(NotARepo.into());
    }

    let output = cmd!("git", "status", "--porcelain"; in repo_dir)
        .context("Unable to check for unsaved changes")?;

    let stdout = String::from_utf8(output.stdout)
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
