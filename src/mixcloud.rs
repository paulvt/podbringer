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

/// A Mixcloud user.
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

/// The Mixcloud cloudcasts container.
#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct CloudcastData {
    /// The contained cloudcasts.
    data: Vec<Cloudcast>,
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

/// Returns the default file type used by Mixcloud.
pub(crate) fn default_file_type() -> &'static str {
    DEFAULT_FILE_TYPE
}

/// Returns the estimated file size in bytes for a given duration.
///
/// This uses the default bitrate (see [`DEFAULT_BITRATE`]) which is in B/s.
pub(crate) fn estimated_file_size(duration: u32) -> u32 {
    DEFAULT_BITRATE * duration / 8
}

/// Retrieves the user data using the Mixcloud API.
#[cached(
    key = "String",
    convert = r#"{ username.to_owned() }"#,
    time = 3600,
    result = true
)]
pub(crate) async fn user(username: &str) -> Result<User> {
    let mut url = Url::parse(API_BASE_URL).expect("URL can always be parsed");
    url.set_path(username);

    println!("⏬ Retrieving user {username} from {url}...");
    let response = reqwest::get(url).await?.error_for_status()?;
    let user = response.json().await?;

    Ok(user)
}

/// Retrieves the cloudcasts of the user using the Mixcloud API.
#[cached(
    key = "String",
    convert = r#"{ username.to_owned() }"#,
    time = 3600,
    result = true
)]
pub(crate) async fn cloudcasts(username: &str) -> Result<Vec<Cloudcast>> {
    let mut url = Url::parse(API_BASE_URL).expect("URL can always be parsed");
    url.set_path(&format!("{username}/cloudcasts/"));

    println!("⏬ Retrieving cloudcasts of user {username} from {url}...");
    let response = reqwest::get(url).await?.error_for_status()?;
    let cloudcasts: CloudcastData = response.json().await?;

    Ok(cloudcasts.data)
}

/// Retrieves the redirect URL for the provided Mixcloud cloudcast key.
#[cached(
    key = "String",
    convert = r#"{ download_key.to_owned() }"#,
    time = 3600,
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
