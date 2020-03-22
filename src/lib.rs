mod providers;

pub use providers::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Repository {
    /// The [`Repository`]'s name.
    pub name: String,
    /// An optional namespace for the [`Repository`], typically used to group
    /// projects by user or organisation.
    pub namespace: Option<String>,
    pub download_info: DownloadInfo,
}

/// Information used to download a [`Repository`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DownloadInfo {
    Git { ssh_url: String },
}
