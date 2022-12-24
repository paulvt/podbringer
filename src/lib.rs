#![doc = include_str!("../README.md")]
#![warn(
    clippy::all,
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    trivial_numeric_casts,
    renamed_and_removed_lints,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![deny(missing_docs)]

use std::path::PathBuf;

use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use rocket::{get, routes, Build, Request, Responder, Rocket, State};
use rocket_dyn_templates::{context, Template};

use crate::backends::Backend;

pub(crate) mod backends;
pub(crate) mod feed;

/// The possible errors that can occur.
#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    /// A standard I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// No redirect URL found in item metadata.
    #[error("No redirect URL found")]
    NoRedirectUrlFound,

    /// A (reqwest) HTTP error occurred.
    #[error("HTTP error: {0}")]
    Request(#[from] reqwest::Error),

    /// Unsupported back-end encountered.
    #[error("Unsupported back-end: {0}")]
    UnsupportedBackend(String),

    /// A URL parse error occurred.
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    /// An error occurred in youtube-dl.
    #[error("Youtube-dl failed: {0}")]
    YoutubeDl(#[from] youtube_dl::Error),

    /// An YouTube extract error occured.
    #[error("YouTube extract error: {0}")]
    YtExtract(#[from] ytextract::Error),

    /// An YouTube extract ID parsing error occured.
    #[error("YouTube extract ID parsing error: {0}")]
    YtExtractId0(#[from] ytextract::error::Id<0>),

    /// An YouTube extract ID parsing error occured.
    #[error("YouTube extract ID parsing error: {0}")]
    YtExtractId11(#[from] ytextract::error::Id<11>),

    /// An YouTube extract ID parsing error occured.
    #[error("YouTube extract ID parsing error: {0}")]
    YtExtractId24(#[from] ytextract::error::Id<24>),

    /// An YouTube extract playlist video error occured.
    #[error("YouTube extract playlist video error: {0}")]
    YtExtractPlaylistVideo(#[from] ytextract::playlist::video::Error),
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for Error {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'o> {
        eprintln!("💥 Encountered error: {}", self);

        match self {
            Error::NoRedirectUrlFound => Err(Status::NotFound),
            _ => Err(Status::InternalServerError),
        }
    }
}

/// Result type that defaults to [`Error`] as the default error type.
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

/// The extra application specific configuration.
#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct Config {
    /// The public URL at which the application is hosted or proxied from.
    #[serde(default)]
    public_url: String,
}

/// A Rocket responder wrapper type for RSS feeds.
#[derive(Responder)]
#[response(content_type = "application/xml")]
struct RssFeed(String);

/// Retrieves a download by redirecting to the URL resolved by the selected back-end.
#[get("/download/<backend_id>/<file..>")]
pub(crate) async fn get_download(file: PathBuf, backend_id: &str) -> Result<Redirect> {
    let backend = backends::get(backend_id)?;

    backend.redirect_url(&file).await.map(Redirect::to)
}

/// Handler for retrieving the RSS feed of a channel on a certain back-end.
///
/// The limit parameter determines the maximum of items that can be in the feed.
#[get("/feed/<backend_id>/<channel_id>?<limit>")]
async fn get_feed(
    backend_id: &str,
    channel_id: &str,
    limit: Option<usize>,
    config: &State<Config>,
) -> Result<RssFeed> {
    let backend = backends::get(backend_id)?;
    let channel = backend.channel(channel_id, limit).await?;
    let feed = feed::construct(backend_id, config, channel);

    Ok(RssFeed(feed.to_string()))
}

/// Returns a simple index page that explains the usage.
#[get("/")]
pub(crate) async fn get_index(config: &State<Config>) -> Template {
    Template::render("index", context! { url: &config.public_url })
}

/// Sets up Rocket.
pub fn setup() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![get_download, get_feed, get_index])
        .attach(AdHoc::config::<Config>())
        .attach(Template::fairing())
}
