// #![doc = include_str!("../README.md")]
#![warn(
    clippy::all,
    missing_debug_implementations,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links
)]
// #![deny(missing_docs)]

use std::process::Stdio;

use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::Url;
use rocket::fairing::AdHoc;
use rocket::response::stream::ReaderStream;
use rocket::serde::{Deserialize, Serialize};
use rocket::{get, routes, Build, Responder, Rocket, State};
use rss::extension::itunes::ITunesItemExtensionBuilder;
use rss::{
    CategoryBuilder, ChannelBuilder, EnclosureBuilder, GuidBuilder, ImageBuilder, ItemBuilder,
};
use tokio::process::{ChildStdout, Command};

pub(crate) mod mixcloud;

/// The extra application specific configuration.
#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct Config {
    #[serde(default)]
    url: String,
}

/// A Rocket responder wrapper type for RSS feeds.
#[derive(Responder)]
#[response(content_type = "application/xml")]
struct RssFeed(String);

/// Handler for retrieving the RSS feed of an Mixcloud user.
#[get("/<username>")]
async fn feed(username: &str, config: &State<Config>) -> Option<RssFeed> {
    let user = mixcloud::get_user(username).await?;
    let cloudcasts = mixcloud::get_cloudcasts(username).await?;
    let mut last_build = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc);
    let generator = String::from(concat!(
        env!("CARGO_PKG_NAME"),
        " ",
        env!("CARGO_PKG_VERSION")
    ));

    let items = cloudcasts
        .into_iter()
        .map(|cloudcast| {
            let slug = cloudcast.slug;
            let mut url = Url::parse(&config.url).unwrap();
            url.set_path(&format!("{}/download", &url.path()[1..]));
            url.query_pairs_mut().append_pair("url", &cloudcast.url);
            let description = format!("Taken from Mixcloud: <{}>", cloudcast.url);
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

            let enclosure = EnclosureBuilder::default()
                .url(url)
                .length(format!(
                    "{}",
                    mixcloud::estimated_file_size(cloudcast.audio_length)
                ))
                .mime_type(String::from(mixcloud::default_file_type()))
                .build();
            let guid = GuidBuilder::default().value(slug).permalink(false).build();
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
    let image = ImageBuilder::default()
        .link(user.pictures.large.clone())
        .url(user.pictures.large)
        .build();

    let channel = ChannelBuilder::default()
        .title(&format!("{} via Mixcloud", user.name))
        .link(&user.url)
        .description(&user.biog)
        .last_build_date(Some(last_build.to_rfc2822()))
        .generator(Some(generator))
        .image(Some(image))
        .items(items)
        .build();
    let feed = RssFeed(channel.to_string());

    Some(feed)
}

/// Retrieves a download using youtube-dl.
#[get("/?<url>")]
pub(crate) async fn download(url: &str) -> Option<ReaderStream![ChildStdout]> {
    let parsed_url = Url::parse(url).ok()?;
    let mut cmd = Command::new("youtube-dl");
    cmd.args(&["--output", "-"])
        .arg(parsed_url.as_str())
        .stdout(Stdio::piped());

    println!("▶️  Streaming enclosure from {parsed_url} using youtube-dl...");
    let mut child = cmd.spawn().ok()?;
    let stdout = child.stdout.take()?;

    tokio::spawn(async move {
        let status = child
            .wait()
            .await
            .expect("child process encounterd an error");
        println!("✅ youtube-dl finished with {}", status);
    });

    Some(ReaderStream::one(stdout))
}
/// Sets up Rocket.
pub fn setup() -> Rocket<Build> {
    rocket::build()
        .mount("/mixcloud", routes![feed])
        .mount("/download", routes![download])
        .attach(AdHoc::config::<Config>())
}
