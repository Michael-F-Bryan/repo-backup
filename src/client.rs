use std::path::PathBuf;
use github_rs::client::Github;
use serde_json::value::from_value;

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

        unimplemented!()
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

    fn clone_repo(&self, repo: &Repo) -> Result<()> {
        let location = self.backup_dir.join(&repo.full_name);

        Ok(())
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
            full_name: String::from("Michael-F-Bryan"),
            url: String::from(root),
        }
    }

    /// This relies on having a valid `GITHUB_TOKEN` environment variable. If 
    /// not found, the test will pass, but be skipped.
    #[test]
    fn get_repos_from_github() {
        let (client, _temp) = match authenticated_client() {
            Ok(v) => v,
            Err(_) => return,
        };

        let repos = client.get_owned_repositories().unwrap();

        assert!(repos.len() > 10, "I know I've got at least 10 owned repositories");
    }

    #[test]
    fn clone_a_repo() {
        let temp = TempDir::new("temp").unwrap();
        let client = Client::new("SOME TOKEN", temp.path()).unwrap();

        let repo = dummy_repository();

        assert_eq!(temp.path().join(&repo.full_name).exists(), false);
        client.clone_repo(&repo).unwrap();
        assert_eq!(temp.path().join(&repo.full_name).exists(), true);
        assert_eq!(temp.path().join(&repo.full_name).join("src/bin/main.rs").exists(), true);
    }
}