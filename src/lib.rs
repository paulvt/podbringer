#![doc = include_str!("../README.md")]
#![warn(
    clippy::all,
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    trivial_numeric_casts
)]
#![deny(missing_docs)]

use std::path::PathBuf;

use chrono::{DateTime, NaiveDateTime, Utc};
use rocket::fairing::AdHoc;
use rocket::http::uri::Absolute;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use rocket::{get, routes, uri, Build, Request, Responder, Rocket, State};
use rocket_dyn_templates::{context, Template};
use rss::extension::itunes::{
    ITunesCategoryBuilder, ITunesChannelExtensionBuilder, ITunesItemExtensionBuilder,
};
use rss::{
    CategoryBuilder, ChannelBuilder, EnclosureBuilder, GuidBuilder, ImageBuilder, ItemBuilder,
};

use crate::backends::{mixcloud, Backend};

pub(crate) mod backends;

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
    /// The URL at which the application is hosted or proxied from.
    #[serde(default)]
    url: String,
}

/// A Rocket responder wrapper type for RSS feeds.
#[derive(Responder)]
#[response(content_type = "application/xml")]
struct RssFeed(String);

/// Retrieves a download by redirecting to the URL resolved by the selected back-end.
#[get("/download/<backend>/<file..>")]
pub(crate) async fn download(file: PathBuf, backend: &str) -> Result<Redirect> {
    match backend {
        "mixcloud" => mixcloud::backend()
            .redirect_url(&file)
            .await
            .map(Redirect::to),
        _ => Err(Error::UnsupportedBackend(backend.to_string())),
    }
}

/// Handler for retrieving the RSS feed of a channel on a certain back-end.
///
/// The limit parameter determines the maximum of items that can be in the feed.
#[get("/feed/<backend>/<channel_id>?<limit>")]
async fn feed(
    backend: &str,
    channel_id: &str,
    limit: Option<usize>,
    config: &State<Config>,
) -> Result<RssFeed> {
    let mut last_build = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc);
    let channel = match backend {
        "mixcloud" => mixcloud::backend().channel(channel_id, limit).await?,
        _ => return Err(Error::UnsupportedBackend(backend.to_string())),
    };

    let category = CategoryBuilder::default()
        .name(
            channel
                .categories
                .first()
                .map(Clone::clone)
                .unwrap_or_default(),
        )
        .build();
    let generator = String::from(concat!(
        env!("CARGO_PKG_NAME"),
        " ",
        env!("CARGO_PKG_VERSION")
    ));
    let image = ImageBuilder::default()
        .link(channel.image.clone())
        .url(channel.image.clone())
        .build();
    let items = channel
        .items
        .into_iter()
        .map(|item| {
            let categories = item
                .categories
                .into_iter()
                .map(|(cat_name, cat_url)| {
                    CategoryBuilder::default()
                        .name(cat_name)
                        .domain(Some(cat_url.to_string()))
                        .build()
                })
                .collect::<Vec<_>>();
            let url = uri!(
                Absolute::parse(&config.url).expect("valid URL"),
                download(backend = backend, file = item.enclosure.file)
            );
            let enclosure = EnclosureBuilder::default()
                .url(url.to_string())
                .length(item.enclosure.length.to_string())
                .mime_type(item.enclosure.mime_type)
                .build();
            let guid = GuidBuilder::default()
                .value(item.guid)
                .permalink(false)
                .build();
            let keywords = item.keywords.join(", ");
            let itunes_ext = ITunesItemExtensionBuilder::default()
                .image(Some(item.image.to_string()))
                .duration(item.duration.map(|dur| format!("{dur}")))
                .subtitle(item.description.clone())
                .keywords(Some(keywords))
                .build();

            if item.updated_at > last_build {
                last_build = item.updated_at;
            }

            ItemBuilder::default()
                .title(Some(item.title))
                .link(Some(item.link.to_string()))
                .description(item.description)
                .categories(categories)
                .enclosure(Some(enclosure))
                .guid(Some(guid))
                .pub_date(Some(item.updated_at.to_rfc2822()))
                .itunes_ext(Some(itunes_ext))
                .build()
        })
        .collect::<Vec<_>>();
    let itunes_ext = ITunesChannelExtensionBuilder::default()
        .author(channel.author)
        .categories(
            channel
                .categories
                .into_iter()
                .map(|cat| ITunesCategoryBuilder::default().text(cat).build())
                .collect::<Vec<_>>(),
        )
        .image(Some(channel.image.to_string()))
        .explicit(Some(String::from("no")))
        .summary(Some(channel.description.clone()))
        .build();

    let channel = ChannelBuilder::default()
        .title(channel.title)
        .link(channel.link)
        .description(channel.description)
        .category(category)
        .last_build_date(Some(last_build.to_rfc2822()))
        .generator(Some(generator))
        .image(Some(image))
        .items(items)
        .itunes_ext(Some(itunes_ext))
        .build();
    let feed = RssFeed(channel.to_string());

    Ok(feed)
}

/// Returns a simple index page that explains the usage.
#[get("/")]
pub(crate) async fn index(config: &State<Config>) -> Template {
    Template::render("index", context! { url: &config.url })
}

/// Sets up Rocket.
pub fn setup() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![download, feed, index])
        .attach(AdHoc::config::<Config>())
        .attach(Template::fairing())
}
