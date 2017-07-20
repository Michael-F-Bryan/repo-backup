//! The errors encountered in this crate.

use hyper::StatusCode;
use std::process::Output;
use raw_github::Repo;


error_chain!{
    links {
        DotEnv(::dotenv::Error, ::dotenv::ErrorKind) #[doc = "A wrapper around a dotenv error"];
        GitHub(::github_rs::errors::Error, ::github_rs::errors::ErrorKind) #[doc = "A GitHub API error"];
    }

    errors {
        /// The server responded with a non-successful status code.
        BadResponse(status: StatusCode, msg: String) {
            description("Bad Response")
            display("Bad Response ({}) - {}", status, msg)
        }

        /// Calling a subcommand failed.
        Subcommand(repo: Repo, cmd: String, output: Output) {
            description("Command Failed")
            display("({}) {:?} failed{}", repo.full_name, cmd, match output.status.code() {
                Some(ret) => format!(" with return code {}", ret),
                None => String::new(),
            })
        }
    }
}