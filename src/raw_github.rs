use std::marker::PhantomData;
use github_rs::client::Github;
use serde::Deserialize;
use serde_json::{self, Value};
use hyper::header::{Headers, Link, RelationType};

use errors::*;


#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Repo {
    pub full_name: String,
    pub clone_url: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Summary {
    pub repos: Vec<Repo>,
}


pub struct Paginated<'a, I>
where
    I: Deserialize<'a>,
{
    client: &'a Github,
    _phantom: PhantomData<I>,
    next_endpoint: Option<String>,
}

impl<'a, I> Paginated<'a, I>
where
    for<'de> I: Deserialize<'de>,
{
    pub fn new(client: &'a Github, endpoint: &str) -> Self {
        Paginated {
            client: client,
            _phantom: PhantomData,
            next_endpoint: Some(String::from(endpoint)),
        }
    }

    /// Try to get the next page, if there are no more pages then
    /// return `None`.
    fn try_next(&mut self) -> Result<Option<I>> {
        if self.next_endpoint.is_none() {
            return Ok(None);
        }

        let endpoint = self.next_endpoint.take().unwrap();
        let jason = self.send_request(&endpoint)?;

        self.interpret_response(jason)
    }

    fn set_new_endpoint(&mut self, headers: &Headers) {
        if let Some(link) = headers.get::<Link>() {
            let next = link.values().iter().find(|lv| {
                lv.rel()
                    .map(|rels| rels.contains(&RelationType::Next))
                    .unwrap_or(false)
            });

            self.next_endpoint = next.map(|n| n.link().replace("https://api.github.com/", ""));
        }
    }

    fn send_request(&mut self, endpoint: &str) -> Result<Option<Value>> {
        debug!("Sending request to {:?}", endpoint);
        let (headers, status, jason) = self.client
            .get()
            .custom_endpoint(endpoint)
            .execute()
            .chain_err(|| format!("Couldn't get {}", endpoint))?;

        trace!("Status: {:?}", status);
        trace!("Headers: {:#?}", headers);
        trace!("Response: {:?}", jason);

        self.set_new_endpoint(&headers);

        if !status.is_success() {
            warn!("Request failed with {}", status);

            bail!(ErrorKind::BadResponse(
                status,
                String::from("Request got an erroneous error code"),
            ));
        }

        Ok(jason)
    }

    fn interpret_response(&self, response: Option<Value>) -> Result<Option<I>> {
        if let Some(response) = response {
            let got: I = serde_json::from_value(response).chain_err(
                || "Couldn't understand the response",
            )?;
            Ok(Some(got))
        } else {
            Err("Didn't receive a response from the server".into())
        }
    }
}


impl<'a, I> Iterator for Paginated<'a, I>
where
    for<'de> I: Deserialize<'de>,
{
    type Item = Result<I>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Err(e) => Some(Err(e)),
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None) => None,
        }
    }
}