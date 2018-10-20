use failure::{Error, ResultExt};
use hyperx::header::{Link, LinkValue, RelationType};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, LINK, USER_AGENT};
use reqwest::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::{self, Value};
use std::marker::PhantomData;
use std::vec::IntoIter;

/// A convenient command runner.
///
/// It behaves like the `format!()` macro, then splits the input string up like
/// your shell would before running the command and inspecting its output to
/// ensure everything was successful.
///
/// # Examples
///
/// ```rust,no_run
/// #[macro_use]
/// extern crate repo_backup;
/// # extern crate failure;
/// # #[macro_use]
/// # extern crate log;
///
/// # fn run() -> Result<(), Box<::std::error::Error>> {
/// let some_url = "https://github.com/Michael-F-Bryan/repo-backup";
/// cmd!(in "/path/to/dir/"; "git clone {}", some_url)?;
/// # Ok(())
/// # }
/// # fn main() { run().unwrap() }
/// ```
#[macro_export]
macro_rules! cmd {
    (in $cwd:expr; $format:expr, $arg:expr) => {
        cmd!(in $cwd; format!($format, $arg))
    };
    (in $cwd:expr; $command:expr) => {{
        use ::failure::ResultExt;

        let command = String::from($command);
        trace!("Executing `{}`", command);
        let arguments: Vec<_> = command.split_whitespace().collect();

        let mut cmd_builder = ::std::process::Command::new(&arguments[0]);
        cmd_builder.current_dir($cwd);

        for arg in &arguments[1..] {
            cmd_builder.arg(arg);
        }

        cmd_builder.output()
            .with_context(|_| format!("Unable to execute `{}`. Is {} installed?", command, &arguments[0]))
            .map_err(::failure::Error::from)
            .and_then(|output| {
                // If the command runs then we need to do a bunch of error
                // checking, making sure to let the user know why the command
                // failed along with the command's stdout/stderr
                if output.status.success() {
                    Ok(())
                } else {
                    match output.status.code() {
                        Some(code) => warn!("`{}` failed with return code {}", command, code),
                        None => warn!("`{}` failed", command),
                    }

                    if !output.stderr.is_empty() {
                        debug!("Stderr:");
                        for line in String::from_utf8_lossy(&output.stderr).lines() {
                            debug!("\t{}", line);
                        }
                    }
                    if !output.stdout.is_empty() {
                        debug!("Stdout:");
                        for line in String::from_utf8_lossy(&output.stdout).lines() {
                            debug!("\t{}", line);
                        }
                    }

                    Err(::failure::err_msg(format!("`{}` failed", command)))
                }
            })
    }};
    ($format:expr, $($arg:expr),*) => {
        cmd!(format!($format, $($arg),*))
    };
    ($command:expr) => {
        cmd!(in "."; $command)
    };
}

pub struct Paginated<I>
where
    I: for<'de> Deserialize<'de>,
{
    client: Client,
    token: String,
    _phantom: PhantomData<I>,
    next_endpoint: Option<String>,
    items: IntoIter<I>,
}

impl<I> Paginated<I>
where
    for<'de> I: Deserialize<'de>,
{
    pub fn new(token: &str, endpoint: &str) -> Self {
        Paginated {
            client: Client::new(),
            token: token.to_string(),
            _phantom: PhantomData,
            next_endpoint: Some(String::from(endpoint)),
            items: Vec::new().into_iter(),
        }
    }

    fn send_request(&mut self, endpoint: &str) -> Result<Vec<I>, Error> {
        debug!("Sending request to {:?}", endpoint);

        let request = self
            .client
            .get(endpoint)
            .header(CONTENT_TYPE, "application/json")
            .header(USER_AGENT, "repo-backup")
            .header(ACCEPT, "application/vnd.github.v3+json")
            .header(AUTHORIZATION, format!("token {}", self.token))
            .build()
            .context("Generated invalid request. This is a bug.")?;

        if log_enabled!(::log::Level::Trace) {
            let redacted_header =
                format!("Request Headers {:#?}", request.headers())
                    .replace(&self.token, "...");

            for line in redacted_header.lines() {
                trace!("{}", line);
            }
        }

        let mut response = self
            .client
            .execute(request)
            .context("Unable to send request")?;

        let raw: Value = response.json()?;
        let status = response.status();
        let headers = response.headers();
        debug!("Received response ({})", status);

        if log_enabled!(::log::Level::Trace) {
            for line in format!("Response Headers {:#?}", headers).lines() {
                trace!("{}", line);
            }

            // trace!("Body:");
            // for line in serde_json::to_string_pretty(&raw).unwrap().lines() {
            //     trace!("{}", line);
            // }
        }

        let got = serde_json::from_value(raw)
            .context("Unable to deserialize response")?;

        if let Some(link) = headers
            .get(LINK)
            .and_then(|l| l.to_str().ok())
            .and_then(|l| l.parse().ok())
        {
            self.next_endpoint = next_link(&link).map(|s| s.to_string());
        }

        if !status.is_success() {
            warn!("Request failed with {}", status);

            let err = FailedRequest {
                status: status,
                url: endpoint.to_string(),
            };

            return Err(err.into());
        }

        Ok(got)
    }
}

impl<I> Iterator for Paginated<I>
where
    for<'de> I: Deserialize<'de>,
{
    type Item = Result<I, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_item) = self.items.next() {
            return Some(Ok(next_item));
        }

        if let Some(next_endpoint) = self.next_endpoint.take() {
            match self.send_request(&next_endpoint) {
                Ok(values) => {
                    self.items = values.into_iter();
                    return self.items.next().map(|it| Ok(it));
                }
                Err(e) => {
                    return Some(Err(e));
                }
            }
        }

        None
    }
}
fn next_link(link: &Link) -> Option<&str> {
    link.values()
        .iter()
        .filter_map(|v| if is_next(v) { Some(v) } else { None })
        .map(|v| v.link())
        .next()
}

fn is_next(link_value: &LinkValue) -> bool {
    link_value
        .rel()
        .map(|relations| relations.iter().any(|rel| *rel == RelationType::Next))
        .unwrap_or(false)
}

#[derive(Debug, Clone, PartialEq, Fail)]
#[fail(display = "Request failed with {}", status)]
pub struct FailedRequest {
    status: StatusCode,
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_next_link() {
        let src = r#"<https://api.github.com/user/repos?page=2>; rel="next", <https://api.github.com/user/repos?page=3>; rel="last""#;
        let link: Link = src.parse().unwrap();

        let should_be = "https://api.github.com/user/repos?page=2";
        let got = next_link(&link).unwrap();
        assert_eq!(got, should_be);
    }
}
