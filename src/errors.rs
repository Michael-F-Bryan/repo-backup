use hyper::StatusCode;
use std::process::Output;


error_chain!{
    links {
        DotEnv(::dotenv::Error, ::dotenv::ErrorKind);
        GitHub(::github_rs::errors::Error, ::github_rs::errors::ErrorKind);
    }

    errors {
        BadResponse(status: StatusCode, msg: String) {
            description("Bad Response")
            display("Bad Response ({}) - {}", status, msg)
        }

        Subcommand(cmd: String, output: Output) {
            description("Command Failed")
            display("{:?} failed{}", cmd, match output.status.code() {
                Some(ret) => format!(" with return code {}", ret),
                None => String::new(),
            })
        }
    }
}