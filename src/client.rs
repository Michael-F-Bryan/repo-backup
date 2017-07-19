use std::fs;
use std::path::{Path, PathBuf};
use github_rs::client::Github;
use git2::{Direction, Repository, Remote};

use errors::*;
use raw_github::{Repo, Summary, Paginated};


pub struct Client {
    backup_dir: PathBuf,
    inner: Github,
}

impl Client {
    /// Create a new client using the provided arguments.
    pub fn new<S, P>(token: S, backup_dir: P) -> Result<Client>
    where
        S: AsRef<str>,
        P: Into<PathBuf>,
    {
        let inner = Github::new(token).chain_err(
            || "Couldn't create a GitHub client",
        )?;

        Ok(Client {
            inner: inner,
            backup_dir: backup_dir.into(),
        })
    }

    /// Start up the backup process.
    pub fn run(&self) -> Result<Summary> {
        info!("Searching GitHub for repositories");
        let owned = self.get_owned_repositories()?;

        info!("Starting backups");
        self.do_backups(&owned).chain_err(|| "Backups failed")?;

        Ok(Summary { repos: owned })
    }

    fn do_backups(&self, repos: &[Repo]) -> Result<()> {
        for repo in repos {
            debug!("Backing up {}", repo.full_name);
            self.backup_repo(repo)?;
        }

        Ok(())
    }

    fn get_owned_repositories(&self) -> Result<Vec<Repo>> {
        let paged: Paginated<Vec<Repo>> = Paginated::new(&self.inner, "user/repos");

        let mut owned_repos = Vec::new();

        for page in paged {
            let repos = page?;
            owned_repos.extend(repos);
        }

        debug!("Found {} owned repos", owned_repos.len());

        Ok(owned_repos)
    }

    fn clone_repo(&self, location: &Path, repo: &Repo) -> Result<Repository> {
        debug!("Cloning into {} ({})", repo.clone_url, location.display());
        Repository::clone(&repo.clone_url, location).chain_err(|| "Cloning failed")
    }

    fn backup_repo(&self, repo: &Repo) -> Result<()> {
        let location = self.backup_dir.join(&repo.full_name);

        let repository = if !location.exists() {
            self.clone_repo(&location, repo)?
        } else {
            Repository::open(&location).chain_err(
                || "Couldn't open the git repository",
            )?
        };

        let mut errs = Vec::new();

        let remotes = repository.remotes().chain_err(
            || "Couldn't get the remotes",
        )?;
        for remote_name in remotes.iter().filter_map(|r| r) {
            trace!("Updating remote {} - {}", repo.full_name, remote_name);
            let remote = repository.find_remote(remote_name).unwrap();

            if let Err(e) = self.update_remote(remote) {
                errs.push(e);
            }
        }

        if errs.is_empty() {
            Ok(())
        } else {
            Err(ErrorKind::FailedUpdate(repo.full_name.clone(), errs).into())
        }
    }

    fn update_remote(&self, mut remote: Remote) -> Result<()> {
        remote.connect(Direction::Fetch).chain_err(
            || "Couldn't connect to remote",
        )?;

        remote.fetch(&[], None, None).map_err(|e| e.into())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempdir::TempDir;
    use dotenv;

    /// Try and get a new `Client`, initialized to use a temporary directory
    /// and the API token from the `GITHUB_TOKEN` environment variable (or
    /// dotenv file, if present).
    fn authenticated_client() -> Result<(Client, TempDir)> {
        dotenv::dotenv().ok();

        let token = env::var("GITHUB_TOKEN").chain_err(|| "Couldn't get token")?;

        let temp = TempDir::new("test").unwrap();
        let client = Client::new(token, temp.path())?;

        Ok((client, temp))
    }

    /// Get a "dummy" repository to be cloned (really just the path of the
    /// project repo).
    fn dummy_repository() -> Repo {
        let root = env!("CARGO_MANIFEST_DIR");

        Repo {
            full_name: String::from("Michael-F-Bryan/github-backup"),
            clone_url: String::from(root),
        }
    }

    /// This relies on having a valid `GITHUB_TOKEN` environment variable. If
    /// not found, the test will pass, but be skipped.
    #[test]
    #[ignore]
    fn get_repos_from_github() {
        let (client, _temp) = match authenticated_client() {
            Ok(v) => v,
            Err(_) => return,
        };

        let repos = client.get_owned_repositories().unwrap();

        assert!(
            repos.len() > 10,
            "I know I've got at least 10 owned repositories"
        );
    }

    #[test]
    fn clone_a_repo() {
        let temp = TempDir::new("temp").unwrap();
        let client = Client::new("SOME TOKEN", temp.path()).unwrap();

        let repo = dummy_repository();

        let parent_dir = temp.path().join("Michael-F-Bryan");
        let repo_dir = parent_dir.join("github-backup");

        assert_eq!(parent_dir.exists(), false);
        assert_eq!(repo_dir.exists(), false);

        client.clone_repo(&repo_dir, &repo).unwrap();

        assert_eq!(parent_dir.exists(), true);
        assert_eq!(repo_dir.exists(), true);
        assert_eq!(
            repo_dir.join("src/bin/main.rs").exists(),
            true,
            "some random file"
        );
    }
}