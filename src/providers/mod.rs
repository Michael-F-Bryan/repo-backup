mod gitlab;
mod pagination;

pub use gitlab::{Config as GitlabConfig, Gitlab};

pub(crate) use pagination::{paginated, Page};

use crate::Repository;
use futures::stream::Stream;

/// A source of [`Repositories`][Repository].
pub trait Provider {
    /// A unique name which can be used to differentiate this [`Provider`] from
    /// others.
    fn name(&self) -> &str;

    /// Retrieve a list of all valid [`Repositories`][Repository].
    fn repositories(
        &self,
    ) -> Box<dyn Stream<Item = Result<Repository, FetchError>>>;
}

/// An error that may occur while fetching download targets with
/// [`Provider::repositories()`].
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("Network error")]
    Reqwest(#[source] reqwest::Error),
}

impl From<reqwest::Error> for FetchError {
    fn from(other: reqwest::Error) -> FetchError { FetchError::Reqwest(other) }
}
