//! The Mixcloud back-end.
//!
//! It uses the Mixcloud API to retrieve the feed (user) and items (cloudcasts)).
//! See also: <https://www.mixcloud.com/developers/>

use cached::proc_macro::cached;
use chrono::{DateTime, Utc};
use reqwest::Url;
use rocket::serde::Deserialize;
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

use super::{Error, Result};

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
    pub(crate) url: String,
}

/// A collection of different sizes/variants of a picture.
#[derive(Clone, Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct Pictures {
    /// The large picture of the user.
    pub(crate) large: String,
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
    pub(crate) url: String,

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
    pub(crate) url: String,
}

/// The base URL for the Mixcloud API.
const API_BASE_URL: &str = "https://api.mixcloud.com";

/// The base URL for downloading Mixcloud files.
const FILES_BASE_URL: &str = "https://www.mixcloud.com";

/// The default bitrate used by Mixcloud.
const DEFAULT_BITRATE: u32 = 64 * 1024;

/// The default file (MIME) type used by Mixcloud.
const DEFAULT_FILE_TYPE: &str = "audio/mpeg";

/// The default page size.
const DEFAULT_PAGE_SIZE: usize = 50;

/// Returns the default file type used by Mixcloud.
pub(crate) const fn default_file_type() -> &'static str {
    DEFAULT_FILE_TYPE
}

/// Returns the estimated file size in bytes for a given duration.
///
/// This uses the default bitrate (see [`DEFAULT_BITRATE`]) which is in B/s.
pub(crate) fn estimated_file_size(duration: u32) -> u32 {
    DEFAULT_BITRATE * duration / 8
}

/// Retrieves the user data using the Mixcloud API.
pub(crate) async fn user(username: &str) -> Result<User> {
    let mut url = Url::parse(API_BASE_URL).expect("URL can always be parsed");
    url.set_path(username);

    println!("⏬ Retrieving user {username} from {url}...");
    fetch_user(url).await
}

/// Fetches the user from the URL.
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

/// Retrieves the cloudcasts data of the user using the Mixcloud API.
pub(crate) async fn cloudcasts(username: &str, limit: Option<usize>) -> Result<Vec<Cloudcast>> {
    let mut limit = limit.unwrap_or(DEFAULT_PAGE_SIZE);
    let mut offset = 0;
    let mut url = Url::parse(API_BASE_URL).expect("URL can always be parsed");
    url.set_path(&format!("{username}/cloudcasts/"));
    println!("⏬ Retrieving cloudcasts of user {username} from {url}...");

    set_paging_query(&mut url, limit, offset);
    let mut cloudcasts = Vec::with_capacity(50); // The initial limit
    loop {
        let cloudcasts_res: CloudcastsResponse = fetch_cloudcasts(url).await?;
        let count = cloudcasts_res.items.len();
        cloudcasts.extend(cloudcasts_res.items);

        // Continue onto the next URL in the paging, if there is one.
        limit = limit.saturating_sub(count);
        offset += count;
        match cloudcasts_res.paging.next {
            Some(next_url) => {
                url = Url::parse(&next_url)?;
                set_paging_query(&mut url, limit, offset);
            }
            None => break,
        }

        // We have reached the limit.
        if limit == 0 {
            break;
        }
    }

    Ok(cloudcasts)
}

/// Fetches cloudcasts from the URL.
///
/// If the result is [`Ok`], the cloudcasts will be cached for 24 hours for the given username.
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
#[cached(
    key = "String",
    convert = r#"{ download_key.to_owned() }"#,
    time = 86400,
    result = true
)]
pub(crate) async fn redirect_url(download_key: &str) -> Result<String> {
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
