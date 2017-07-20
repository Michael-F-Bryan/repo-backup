use github_rs::client::Github;
use serde::de::Deserialize;

use errors::*;
use raw_github::{Repo, Paginated};


/// Use the provided API token to find all the repositories the user ownes
/// or has starred.
pub fn get_repos(token: &str) -> Result<Vec<Repo>> {
    info!("Finding repositories to backup");
    let client = Github::new(token)?;

    let mut repositories = owned_repositories(&client)?;
    repositories.extend(starred(&client)?);
    info!("{} repos found", repositories.len());

    for repo in &repositories {
        debug!("{}", repo.full_name);
    }

    Ok(repositories)
}

fn owned_repositories(client: &Github) -> Result<Vec<Repo>> {
    let got = paginated(client, "user/repos")?;
    debug!("Found {} owned repos", got.len());
    Ok(got)
}

fn starred(client: &Github) -> Result<Vec<Repo>> {
    let got = paginated(client, "user/starred")?;
    debug!("Found {} starred repos", got.len());
    Ok(got)
}

/// Send a GET request to the provided endpoint, concatenating the paginated
/// response into a single list of items.
///
/// **Note:** This assumes that the endpoint will give you a list of `T`'s.
fn paginated<'a, T>(client: &'a Github, endpoint: &str) -> Result<Vec<T>>
where
    for<'de> T: Deserialize<'de>,
{
    let paged: Paginated<Vec<T>> = Paginated::new(client, endpoint);
    let mut results = Vec::new();

    for page in paged {
        results.extend(page?);
    }
    Ok(results)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempdir::TempDir;
    use dotenv;

    /// Get an API token from the `GITHUB_TOKEN` environment variable (or a
    /// dotfile).
    fn token() -> Result<String> {
        dotenv::dotenv().ok();
        env::var("GITHUB_TOKEN").chain_err(|| "Couldn't get token")
    }

    /// Get a "dummy" repository to be cloned (really just the path of the
    /// project repo).
    fn dummy_repository() -> Repo {
        let root = env!("CARGO_MANIFEST_DIR");

        Repo {
            name: String::from("github-backup"),
            full_name: String::from("Michael-F-Bryan/github-backup"),
            clone_url: String::from(root),
        }
    }

    /// This relies on having a valid `GITHUB_TOKEN` environment variable. If
    /// not found, the test will pass, but be skipped.
    #[test]
    fn get_repos_from_github() {
        let token = match token() {
            Ok(v) => v,
            Err(_) => return,
        };

        let got = get_repos(&token).unwrap();

        assert!(
            got.len() > 10,
            "I know I've got at least 10 owned repositories..."
        );
    }
}