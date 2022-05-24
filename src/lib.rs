#![doc = include_str!("../README.md")]
#![warn(
    clippy::all,
    missing_debug_implementations,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links
)]
#![deny(missing_docs)]

use std::path::PathBuf;

use chrono::{DateTime, NaiveDateTime, Utc};
use rocket::fairing::AdHoc;
use rocket::http::uri::Absolute;
use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use rocket::{get, routes, uri, Build, Responder, Rocket, State};
use rocket_dyn_templates::{context, Template};
use rss::extension::itunes::{
    ITunesCategoryBuilder, ITunesChannelExtensionBuilder, ITunesItemExtensionBuilder,
};
use rss::{
    CategoryBuilder, ChannelBuilder, EnclosureBuilder, GuidBuilder, ImageBuilder, ItemBuilder,
};

pub(crate) mod mixcloud;

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

/// Retrieves a download using youtube-dl and redirection.
#[get("/download/<backend>/<file..>")]
pub(crate) async fn download(file: PathBuf, backend: &str) -> Option<Redirect> {
    match backend {
        "mixcloud" => {
            let key = format!("/{}/", file.with_extension("").to_string_lossy());

            mixcloud::redirect_url(&key).await.map(Redirect::to)
        }
        _ => None,
    }
}

/// Handler for retrieving the RSS feed of an Mixcloud user.
#[get("/feed/<backend>/<username>")]
async fn feed(backend: &str, username: &str, config: &State<Config>) -> Option<RssFeed> {
    let user = mixcloud::get_user(username).await?;
    let cloudcasts = mixcloud::get_cloudcasts(username).await?;
    let mut last_build = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc);

    let category = CategoryBuilder::default()
        .name(String::from("Music")) // FIXME: Don't hardcode the category!
        .build();
    let generator = String::from(concat!(
        env!("CARGO_PKG_NAME"),
        " ",
        env!("CARGO_PKG_VERSION")
    ));
    let image = ImageBuilder::default()
        .link(user.pictures.large.clone())
        .url(user.pictures.large.clone())
        .build();
    let items = cloudcasts
        .into_iter()
        .map(|cloudcast| {
            let mut file = PathBuf::from(cloudcast.key.trim_end_matches('/'));
            file.set_extension("m4a"); // FIXME: Don't hardcode the extension!
            let url = uri!(
                Absolute::parse(&config.url).expect("valid URL"),
                download(backend = backend, file = file)
            );
            // FIXME: Don't hardcode the description!
            let description = format!("Taken from Mixcloud: {}", cloudcast.url);
            let keywords = cloudcast
                .tags
                .iter()
                .map(|tag| &tag.name)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            let categories = cloudcast
                .tags
                .into_iter()
                .map(|tag| {
                    CategoryBuilder::default()
                        .name(tag.name)
                        .domain(Some(tag.url))
                        .build()
                })
                .collect::<Vec<_>>();

            let length = mixcloud::estimated_file_size(cloudcast.audio_length);
            let enclosure = EnclosureBuilder::default()
                .url(url.to_string())
                .length(format!("{}", length))
                .mime_type(String::from(mixcloud::default_file_type()))
                .build();
            let guid = GuidBuilder::default()
                .value(cloudcast.slug)
                .permalink(false)
                .build();
            let itunes_ext = ITunesItemExtensionBuilder::default()
                .image(Some(cloudcast.pictures.large))
                .duration(Some(format!("{}", cloudcast.audio_length)))
                .subtitle(Some(description.clone()))
                .keywords(Some(keywords))
                .build();

            if cloudcast.updated_time > last_build {
                last_build = cloudcast.updated_time;
            }

            ItemBuilder::default()
                .title(Some(cloudcast.name))
                .link(Some(cloudcast.url))
                .description(Some(description))
                .categories(categories)
                .enclosure(Some(enclosure))
                .guid(Some(guid))
                .pub_date(Some(cloudcast.updated_time.to_rfc2822()))
                .itunes_ext(Some(itunes_ext))
                .build()
        })
        .collect::<Vec<_>>();
    let itunes_ext = ITunesChannelExtensionBuilder::default()
        .author(Some(user.name.clone()))
        .categories(Vec::from([ITunesCategoryBuilder::default()
            .text("Music")
            .build()])) // FIXME: Don't hardcode the category!
        .image(Some(user.pictures.large))
        .explicit(Some(String::from("no")))
        .summary(Some(user.biog.clone()))
        .build();

    let channel = ChannelBuilder::default()
        .title(&format!("{} (via Mixcloud)", user.name))
        .link(&user.url)
        .description(&user.biog)
        .category(category)
        .last_build_date(Some(last_build.to_rfc2822()))
        .generator(Some(generator))
        .image(Some(image))
        .items(items)
        .itunes_ext(Some(itunes_ext))
        .build();
    let feed = RssFeed(channel.to_string());

    Some(feed)
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
