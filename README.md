# github-backup
A small tool for backing up your GitHub repos to a local directory.

The program will:

- Get a list of all owned and starred repos using the GitHub API, and
- For each repo,
  - If the `<backup_dir>/<user>/<repo>` directory doesn't exist, clone it
  - Otherwise `cd` into the directory and run `git pull` and update any
    git submodules (if applicable)


## Usage

First you'll need to install the program using `cargo`.

```bash
$ cargo install --git https://github.com/Michael-F-Bryan/github-backup
```

> **Note:** You'll need to install directly from git for now. I haven't
> decided whether it's worth uploading to `crates.io` yet.

Then run it in your chosen directory:

```bash
$ cd ~/github-backups
$ github-backup
```

Alternatively you can use the `-d` flag to specify a backup directory:

```bash
$ github-backup -d /tmp
```

This uses the GitHub API, so you'll need to make sure you [create a personal
access token]token] and save it either as the `GITHUB_TOKEN` environment 
variable or in a `.env` file so [dotenv] can find it (I usually use `~/.env`).

The tool tries to be quiet by default, however you can keep adding `-v` 
arguments to make it successively more verbose.


```bash
$ github-backup --help
github-backup 0.1.0
Michael Bryan <michaelfbryan@gmail.com>
A program for backing up your GitHub repos

USAGE:
    github-backup [FLAGS] [OPTIONS]

FLAGS:
    -h, --help          Prints help information
    -s, --sequential    Run the backups sequentially (default is in parallel)
    -V, --version       Prints version information
    -v, --verbose       Sets the verbosity level (repeat for more verbosity)

OPTIONS:
    -d, --backup-dir <backup-dir>    The directory to save backups to. [default: .]
    -t, --token <token>              Your GitHub API token (defaults to GITHUB_TOKEN env variable)
```


[dotenv]: https://docs.rs/dotenv
[token]: https://github.com/settings/tokens/new