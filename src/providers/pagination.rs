use crate::{providers::FetchError, Repository};
use futures::{future::Future, stream::Stream};
use reqwest::{header::HeaderMap, Client, Response, Url};
use async_stream::try_stream;

pub(crate) fn paginated<P, F>(
    client: Client,
    first_page: Url,
    headers: HeaderMap,
    mut parse: P,
) -> impl Stream<Item = Result<Repository, FetchError>>
where
    P: FnMut(Response) -> F,
    F: Future<Output = Result<Page, FetchError>>,
{
    try_stream! {
        let mut page = Some(first_page);

        while let Some(next_page) = page {
            let response = client.get(next_page).headers(headers.clone()).send().await?;
            let Page { next, repositories} = parse(response).await?;

            for repo in repositories {
                yield repo;
            }
            page = next;
        }
    }
}

pub(crate) struct Page {
    pub next: Option<Url>,
    pub repositories: Vec<Repository>,
}
