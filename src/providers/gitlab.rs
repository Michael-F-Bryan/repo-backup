use crate::{
    providers::{paginated, FetchError, Provider},
    Repository,
};
use futures::stream::Stream;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client, Url,
};

/// A Gitlab client which can be used as a [`Provider`].
#[derive(Debug, Clone)]
pub struct Gitlab {
    client: Client,
    base_url: Url,
    auth: Auth,
}

impl Gitlab {
    pub fn with_personal_access_token(
        client: Client,
        base_url: Url,
        token: String,
    ) -> Self {
        Gitlab {
            client,
            base_url,
            auth: Auth::Token(token),
        }
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        match self.auth {
            Auth::Token(ref token) => {
                let v = HeaderValue::from_str(&token).expect("Tokens");
                headers.insert(AUTHORIZATION, v);
            },
        }

        headers
    }

    pub fn owned_repositories(
        &self,
    ) -> impl Stream<Item = Result<Repository, FetchError>> {
        let first_page = self.base_url.clone();
        paginated(self.client.clone(), first_page, self.headers(), |r| async {
            let _bytes = r.bytes().await;

            unimplemented!()
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Auth {
    Token(String),
}

impl Provider for Gitlab {
    fn name(&self) -> &str { self.base_url.host_str().unwrap_or("gitlab.com") }

    /// Retrieve a list of all valid [`Repositories`][Repository].
    fn repositories(
        &self,
    ) -> Box<dyn Stream<Item = Result<Repository, FetchError>>> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub host: Url,
    pub token: String,
}