# github-backup
A small tool for downloading all your GitHub repositories locally.


## Usage

First you'll need to install the program using `cargo`.

```bash
$ cargo install --git https://github.com/Michael-F-Bryan/github-backup
```

> **Note:** You'll need to install directly from git for now, I haven't
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
```