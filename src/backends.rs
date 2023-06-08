//! The supported content back-ends.
//!
//! A content back-end should provide two kinds of objects: channels and their (content) items.
//! It must provide a methods to retrieve a channel and its items and a method to return the
//! redirect URL for some path that points to media within context of the back-end.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use enum_dispatch::enum_dispatch;
use reqwest::Url;

use crate::{Error, Result};

pub(crate) mod mixcloud;
pub(crate) mod youtube;

/// Retrieves the back-end for the provided ID (if supported).
pub(crate) fn get(backend: &str) -> Result<Backends> {
    match backend {
        "mixcloud" => Ok(Backends::Mixcloud(mixcloud::backend())),
        "youtube" => Ok(Backends::YouTube(youtube::backend())),
        _ => Err(Error::UnsupportedBackend(backend.to_string())),
    }
}

/// The supported back-ends.
#[enum_dispatch(Backend)]
pub(crate) enum Backends {
    /// Mixcloud (<https://www.mixcloud.com>)
    Mixcloud(mixcloud::Backend),

    /// YouTube (<https://www.youtube.com>)
    YouTube(youtube::Backend),
}

/// Functionality of a content back-end.
#[async_trait]
#[enum_dispatch]
pub(crate) trait Backend {
    /// Returns the name of the backend.
    fn name(&self) -> &'static str;

    /// Returns the channel with its currently contained content items.
    async fn channel(&self, channel_id: &str, item_limit: Option<usize>) -> Result<Channel>;

    /// Returns the redirect URL for the provided download file path.
    async fn redirect_url(&self, file: &Path) -> Result<String>;
}

/// The metadata of a collection of content items.
#[derive(Clone, Debug)]
pub(crate) struct Channel {
    /// The title of the channel.
    pub(crate) title: String,

    /// The link to the channel.
    pub(crate) link: Url,

    /// The description of the channel.
    pub(crate) description: String,

    /// The author/composer/creator of the channel.
    pub(crate) author: Option<String>,

    /// The categories associated with the channel.
    ///
    /// The first category is considered to be the "main" category.
    pub(crate) categories: Vec<String>,

    /// The URL of the image/logo/avatar of a channel.
    pub(crate) image: Option<Url>,

    /// The contained content items.
    pub(crate) items: Vec<Item>,
}

/// A content item belonging to a channel.
#[derive(Clone, Debug)]
pub(crate) struct Item {
    /// The title of the item.
    pub(crate) title: String,

    /// The direct link to the item.
    pub(crate) link: Url,

    /// The description of the item.
    pub(crate) description: Option<String>,

    /// The categories of the items (and their domain URLs).
    pub(crate) categories: HashMap<String, Url>,

    /// The enclosed media content of the item,
    pub(crate) enclosure: Enclosure,

    /// The duration of the media content (in seconds).
    pub(crate) duration: Option<u32>,

    /// The global UID of the item.
    ///
    /// This GUID is not considered nor needs to be a permalink.
    pub(crate) guid: String,

    /// The keywords associated with the item.
    pub(crate) keywords: Vec<String>,

    /// The URL of the image of the item.
    pub(crate) image: Option<Url>,

    /// The timestamp the item was published.
    pub(crate) published_at: DateTime<Utc>,

    /// The timestamp the item was last updated.
    pub(crate) updated_at: DateTime<Utc>,
}

/// The enclosed media content of an item.
#[derive(Clone, Debug)]
pub(crate) struct Enclosure {
    /// The path of the download file associated with the item enclosure.
    ///
    /// This is used as a part of the enclosure URL of the item and will be passed to
    /// [`Backend::redirect_url`] later when a client wants to download the media content.
    pub(crate) file: PathBuf,

    /// The MIME type of the download file path associated with the item enclosure.
    pub(crate) mime_type: String,

    /// The length of the enclosed media content (in bytes).
    pub(crate) length: u64,
}
