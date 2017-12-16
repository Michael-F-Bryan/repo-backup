use std::marker::PhantomData;
use std::vec::IntoIter;
use serde::Deserialize;
use serde_json::{self, Value};
use reqwest::Client;
use reqwest::StatusCode;
use reqwest::header::{qitem, Accept, Authorization, ContentType, Link, LinkValue, RelationType,
                      UserAgent};
use failure::{Error, ResultExt};

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

        let mime_type = "application/vnd.github.v3+json".parse()?;
        let request = self.client
            .get(endpoint)
            .header(ContentType::json())
            .header(UserAgent::new(String::from("repo-backup")))
            .header(Accept(vec![qitem(mime_type)]))
            .header(Authorization(format!("token {}", self.token)))
            .build()
            .context("Generated invalid request. This is a bug.")?;

        if log_enabled!(::log::Level::Trace) {
            let redacted_header = format!("Request Headers {:#?}", request.headers())
                .replace(&self.token, "XXXXXXXXXX");

            for line in redacted_header.lines() {
                trace!("{}", line);
            }
        }

        let mut response = self.client
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

            trace!("Body:");
            for line in serde_json::to_string_pretty(&raw).unwrap().lines() {
                trace!("{}", line);
            }
        }

        let got = serde_json::from_value(raw).context("Unable to deserialize response")?;

        if let Some(link) = headers.get::<Link>() {
            self.next_endpoint = next_link(link).map(|s| s.to_string());
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

#[derive(Debug, Clone, PartialEq, Fail)]
#[fail(display = "Request failed with {}", status)]
pub struct FailedRequest {
    status: StatusCode,
    url: String,
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
            // send request
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
