# Repo Backup

[![Build Status](https://travis-ci.org/Michael-F-Bryan/repo-backup.svg?branch=master)](https://travis-ci.org/Michael-F-Bryan/repo-backup)
[![Build status](https://ci.appveyor.com/api/projects/status/9ik2qiov3l2buyqd?svg=true)](https://ci.appveyor.com/project/Michael-F-Bryan/repo-backup)
[![Crates.io](https://img.shields.io/crates/v/repo-backup.svg)](https://crates.io/crates/repo-backup)
[![Docs](https://docs.rs/repo-backup/badge.svg)](https://docs.rs/repo-backup)
![License](https://img.shields.io/github/license/Michael-F-Bryan/repo-backup.svg)


A small utility for making a local copy of all your projects from a variety
of various sources.

Sources currently supported:

<table>
    <thead>
        <tr>
            <th>Provider</th>
            <th>Available Repositories</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>
                <a href="https://github.com"/>GitHub</a>
            </td>
            <td>
                <ul>
                    <li>owned</li>
                    <li>starred</li>
                </ul>
            </td>
        </tr>
        <tr>
            <td>
                <a href="https://about.gitlab.com"/>GitLab</a>
            </td>
            <td>
                <ul>
                    <li>owned</li>
                    <li>repositories belonging to organisations you are a part of</li>
                </ul>
            </td>
        </tr>
    </tbody>
</table>


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
2017-12-17 02:01:42 [INFO ]: Starting repository backup
2017-12-17 02:01:42 [INFO ]: Fetching repositories from github
2017-12-17 02:01:49 [INFO ]: Found 209 repos from github
2017-12-17 02:01:49 [INFO ]: Updating repositories
2017-12-17 02:05:46 [INFO ]: Finished repository backup
```

This tool is designed to avoid superfluous output and only print messages to
the terminal when there is an issue (sometimes known as ["the rule of silence"]
in the *Unix Philosophy*). However, you can tell it to be more verbose by
adding consecutively more `-v` flags.

The generated tree structure looks something like this (with a couple hundred
directories elided for conciseness):

```
$ tree -L 3 /srv/
/srv/
├── github
│   ├── BurntSushi
│   │   └── ripgrep
    ...
│   ├── Michael-F-Bryan
│   │   ├── rust-ffi-guide
    ...
│   │   └── repo-backup
    ...
│   └── yupferris
│       └── rustendo64
└── gitlab
    ├── Curtin-Motorsport-Team
    │   ├── CAN-node
    ...
    │   └── telemetry
    └── Michael-F-Bryan
        ├── dotfiles
    ...
        └── uni-work
```


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
> 
> [gitlab]
> api-key = 'your API key'
> url = 'https://gitlab.com/'
> organisations = true
> owned = true
> ```
>
> In general, all `Provider` specific keys are optional, with the exception of
> an `api-key`.

[GitHub Releases]: https://github.com/Michael-F-Bryan/repo-backup/releases
[Rust]: https://www.rust-lang.org/en-US/
["the rule of silence"]: http://www.linfo.org/rule_of_silence.html