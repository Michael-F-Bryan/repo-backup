# github-backup
A small tool for downloading all your GitHub repositories locally.


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