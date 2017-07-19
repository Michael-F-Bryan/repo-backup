use hyper::StatusCode;


error_chain!{
    links {
        DotEnv(::dotenv::Error, ::dotenv::ErrorKind);
        GitHub(::github_rs::errors::Error, ::github_rs::errors::ErrorKind);
    }

    foreign_links {
        Git(::git2::Error);
    }

    errors {
        BadResponse(status: StatusCode, msg: String) {
            description("Bad Response")
            display("Bad Response ({}) - {}", status, msg)
        }
        FailedUpdate(repo: String, errs: Vec<Error>) {
            description("Failed Update")
            display("Failed Update for {} ({})", repo, 
                errs.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join(", "))
        }
    }
}