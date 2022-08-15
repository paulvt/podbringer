//! The Mixcloud back-end.
//!
//! It uses the Mixcloud API to retrieve the feed (user) and items (cloudcasts)).
//! See also: <https://www.mixcloud.com/developers/>

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use cached::proc_macro::cached;
use chrono::{DateTime, Utc};
use reqwest::Url;
use rocket::serde::Deserialize;
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

use super::{Channel, Enclosure, Item};
use crate::{Error, Result};

/// The base URL for the Mixcloud API.
const API_BASE_URL: &str = "https://api.mixcloud.com";

/// The base URL for downloading Mixcloud files.
const FILES_BASE_URL: &str = "https://www.mixcloud.com";

/// The default bitrate used by Mixcloud.
const DEFAULT_BITRATE: u64 = 64 * 1024;

/// The default file (MIME) type used by Mixcloud.
const DEFAULT_FILE_TYPE: &str = "audio/mpeg";

/// The default page size.
const DEFAULT_PAGE_SIZE: usize = 50;

/// Creates a Mixcloud back-end.
pub(crate) fn backend() -> Backend {
    Backend
}

/// The Mixcloud back-end.
pub struct Backend;

#[async_trait]
impl super::Backend for Backend {
    fn name(&self) -> &'static str {
        "Mixcloud"
    }

    async fn channel(&self, channel_id: &str, item_limit: Option<usize>) -> Result<Channel> {
        // For Mixcloud a channel ID is some user name.
        let mut user_url = Url::parse(API_BASE_URL).expect("URL can always be parsed");
        user_url.set_path(channel_id);

        println!("⏬ Retrieving user {channel_id} from {user_url}...");
        let user = fetch_user(user_url).await?;

        // The items of a channel are the user's cloudcasts.
        let mut limit = item_limit.unwrap_or(DEFAULT_PAGE_SIZE);
        let mut offset = 0;
        let mut cloudcasts_url = Url::parse(API_BASE_URL).expect("URL can always be parsed");
        cloudcasts_url.set_path(&format!("{channel_id}/cloudcasts/"));
        println!("⏬ Retrieving cloudcasts of user {channel_id} from {cloudcasts_url}...");

        set_paging_query(&mut cloudcasts_url, limit, offset);
        let mut cloudcasts = Vec::with_capacity(50); // The initial limit
        loop {
            let cloudcasts_res: CloudcastsResponse = fetch_cloudcasts(cloudcasts_url).await?;
            let count = cloudcasts_res.items.len();
            cloudcasts.extend(cloudcasts_res.items);

            // Continue onto the next URL in the paging, if there is one and the limit was not
            // reached.
            limit = limit.saturating_sub(count);
            offset += count;
            match (limit, cloudcasts_res.paging.next) {
                (0, Some(_)) => break,
                (_, Some(next_url)) => {
                    cloudcasts_url = Url::parse(&next_url)?;
                    set_paging_query(&mut cloudcasts_url, limit, offset);
                }
                (_, None) => break,
            }
        }

        Ok(Channel::from(UserWithCloudcasts(user, cloudcasts)))
    }

    async fn redirect_url(&self, file: &Path) -> Result<String> {
        let key = format!("/{}/", file.with_extension("").to_string_lossy());

        retrieve_redirect_url(&key).await
    }
}

/// A Mixcloud user with its cloudcasts.
pub(crate) struct UserWithCloudcasts(User, Vec<Cloudcast>);

/// A Mixcloud user (response).
#[derive(Clone, Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct User {
    /// The name of the user.
    pub(crate) name: String,

    /// The bio (description) of the user.
    pub(crate) biog: String,

    /// The picture URLs associated with the user.
    pub(crate) pictures: Pictures,

    /// The original URL of the user.
    pub(crate) url: Url,
}

/// A collection of different sizes/variants of a picture.
#[derive(Clone, Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct Pictures {
    /// The URL of a large picture of the user.
    pub(crate) large: Url,
}

/// The Mixcloud cloudcasts response.
#[derive(Clone, Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct CloudcastsResponse {
    /// The contained cloudcast items.
    #[serde(rename = "data")]
    items: Vec<Cloudcast>,

    /// The paging information.
    paging: CloudcastsPaging,
}

/// The Mixcloud paging info.
#[derive(Clone, Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct CloudcastsPaging {
    /// The API URL of the next page.
    next: Option<String>,
}

/// A Mixcloud cloudcast.
#[derive(Clone, Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct Cloudcast {
    /// The key of the cloudcast.
    pub(crate) key: String,

    /// The name of the cloudcast.
    pub(crate) name: String,

    /// The slug of the cloudcast (used for the enclosure).
    pub(crate) slug: String,

    /// The picture URLs associated with the cloudcast.
    pub(crate) pictures: Pictures,

    /// The tags of the cloudcast.
    pub(crate) tags: Vec<Tag>,

    /// The time the feed was created/started.
    pub(crate) updated_time: DateTime<Utc>,

    /// The original URL of the cloudcast.
    pub(crate) url: Url,

    /// The length of the cloudcast (in seconds).
    pub(crate) audio_length: u32,
}

/// A Mixcloud cloudcast tag.
#[derive(Clone, Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct Tag {
    /// The name of the tag.
    pub(crate) name: String,

    /// The URL of the tag.
    pub(crate) url: Url,
}

impl From<UserWithCloudcasts> for Channel {
    fn from(UserWithCloudcasts(user, cloudcasts): UserWithCloudcasts) -> Self {
        // FIXME: Don't hardcode the category!
        let categories = Vec::from([String::from("Music")]);
        let items = cloudcasts.into_iter().map(From::from).collect();

        Channel {
            title: format!("{0} (via Mixcloud)", user.name),
            link: user.url,
            description: user.biog,
            author: Some(user.name),
            categories,
            image: Some(user.pictures.large),
            items,
        }
    }
}

impl From<Cloudcast> for Item {
    fn from(cloudcast: Cloudcast) -> Self {
        let mut file = PathBuf::from(cloudcast.key.trim_end_matches('/'));
        file.set_extension("m4a"); // FIXME: Don't hardcoded the extension!

        // FIXME: Don't hardcode the description!
        let description = Some(format!("Taken from Mixcloud: {0}", cloudcast.url));
        let categories = cloudcast
            .tags
            .iter()
            .cloned()
            .map(|tag| (tag.name, tag.url))
            .collect();
        let enclosure = Enclosure {
            file,
            mime_type: String::from(DEFAULT_FILE_TYPE),
            length: estimated_file_size(cloudcast.audio_length),
        };
        let keywords = cloudcast.tags.into_iter().map(|tag| tag.name).collect();

        Item {
            title: cloudcast.name,
            link: cloudcast.url,
            description,
            categories,
            enclosure,
            duration: Some(cloudcast.audio_length),
            guid: cloudcast.slug,
            keywords,
            image: Some(cloudcast.pictures.large),
            updated_at: cloudcast.updated_time,
        }
    }
}

/// Returns the estimated file size in bytes for a given duration.
///
/// This uses the default bitrate (see [`DEFAULT_BITRATE`]) which is in B/s.
fn estimated_file_size(duration: u32) -> u64 {
    DEFAULT_BITRATE * duration as u64 / 8
}

/// Fetches the user from the URL.
///
/// If the result is [`Ok`], the user will be cached for 24 hours for the given URL.
#[cached(
    key = "String",
    convert = r#"{ url.to_string() }"#,
    time = 86400,
    result = true
)]
///
/// If the result is [`Ok`], the user will be cached for 24 hours for the given username.
async fn fetch_user(url: Url) -> Result<User> {
    let response = reqwest::get(url).await?.error_for_status()?;
    let user = response.json().await?;

    Ok(user)
}

/// Fetches cloudcasts from the URL.
///
/// If the result is [`Ok`], the cloudcasts will be cached for 24 hours for the given URL.
#[cached(
    key = "String",
    convert = r#"{ url.to_string() }"#,
    time = 86400,
    result = true
)]
async fn fetch_cloudcasts(url: Url) -> Result<CloudcastsResponse> {
    let response = reqwest::get(url).await?.error_for_status()?;
    let cloudcasts_res = response.json().await?;

    Ok(cloudcasts_res)
}

/// Set paging query pairs for URL.
///
/// The limit is capped to the default page size. Another request will be necessary to retrieve
/// more.
fn set_paging_query(url: &mut Url, limit: usize, offset: usize) {
    url.query_pairs_mut()
        .clear()
        .append_pair(
            "limit",
            &format!("{}", std::cmp::min(limit, DEFAULT_PAGE_SIZE)),
        )
        .append_pair("offset", &format!("{}", offset));
}

/// Retrieves the redirect URL for the provided Mixcloud cloudcast key.
///
/// If the result is [`Ok`], the redirect URL will be cached for 24 hours for the given cloudcast
/// key.
#[cached(
    key = "String",
    convert = r#"{ download_key.to_owned() }"#,
    time = 86400,
    result = true
)]
async fn retrieve_redirect_url(download_key: &str) -> Result<String> {
    let mut url = Url::parse(FILES_BASE_URL).expect("URL can always be parsed");
    url.set_path(download_key);

    println!("🌍 Determining direct URL for {download_key}...");
    let output = YoutubeDl::new(url).run_async().await?;

    if let YoutubeDlOutput::SingleVideo(yt_item) = output {
        yt_item.url.ok_or(Error::NoRedirectUrlFound)
    } else {
        Err(Error::NoRedirectUrlFound)
    }
}
