# Repo Backup

[![Build Status](https://travis-ci.org/Michael-F-Bryan/repo-backup.svg?branch=master)](https://travis-ci.org/Michael-F-Bryan/repo-backup)
[![Build status](https://ci.appveyor.com/api/projects/status/9ik2qiov3l2buyqd?svg=true)](https://ci.appveyor.com/project/Michael-F-Bryan/repo-backup)


A small utility for making a local copy of all your projects from a variety
of various sources.

Sources currently supported:

- [GitHub](https://github.com/)


## Getting Started

If you already have [Rust] installed, you can install the program directly from
crates.io:

```
$ cargo install repo-backup
```

Otherwise, pre-compiled binaries are available from [GitHub Releases].

Once you have installed `repo-backup`, you can run it from the command line.

```
$ repo-backup -v
2017-12-17 02:01:42 [INFO ] (repo_backup::driver#28): Starting repository backup
2017-12-17 02:01:42 [INFO ] (repo_backup::driver#79): Fetching repositories from github
2017-12-17 02:01:49 [INFO ] (repo_backup::driver#84): Found 209 repos from github
2017-12-17 02:01:49 [INFO ] (repo_backup::driver#40): Updating repositories
2017-12-17 02:05:46 [INFO ] (repo_backup::driver#34): Finished repository backup
```

Following [The Unix Philosophy], this tool is designed to avoid superfluous
output and only print messages to the terminal when there is an issue. However,
you can tell it to be more verbose by adding consecutively more `-v` flags.


## Configuration

Configuration is done via a `repo-backup.toml` file. By default the
`repo-backup` program will look for this in your home directory (as
`~/.repo-backup.toml`), but this can be overridden via the command line.

The configuration file looks something like this:

```toml
[general]
dest-dir = "/srv"

[github]
api-key = "your API key"
owned = true
starred = false
```

The only required table is `general`, with the others used to enable and
configure the corresponding `Provider`.

> **Hint:** You can ask the tool to print an example config using the 
> `--example-config` flag.
> 
> ```
> $ repo-backup --example-config
> [general]
> dest-dir = '/srv'
> 
> [github]
> api-key = 'your API key'
> starred = false
> owned = true
> ```



[GitHub Releases]: https://github.com/Michael-F-Bryan/repo-backup/releases
[Rust]: https://www.rust-lang.org/en-US/