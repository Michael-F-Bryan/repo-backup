# Changelog

## Unreleased

### Added

- Added a GitLab `Provider` for fetching owned repositories and those belonging
  to organisations you are a part of
- Improved error handling with the `failure` crate
- API keys are now hidden from the logs
- This `CHANGELOG`

### Changed 

- Logging no longer prints the module and line number 


## 0.2.0 (2017-12-17)

This was essentially a rewrite of the project to decouple a source of 
repositories (a `Provider`) from the list of repositories being downloaded.

### Added

- Configuration is done via a `~/.repo-backup.toml` config file
- Repository sources are represented by the `Provider` trait
- Added a GitHub `Provider` for fetching owned and starred repositories
- You can use the `--example-config` flag to generate an example config file


## 0.1.0 (2017-07-20)

### Added

- Now able to download repositories via `git clone`
- Repositories are updated with a `git pull`
- The GitHub API is used to fetch a list of owned and starred repositories 
  (requires an API token)
- `dotenv` and the `GITHUB_TOKEN` environment variable can be used to pass in a
  GitHub API token