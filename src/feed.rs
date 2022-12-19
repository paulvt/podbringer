//! Helper functions for constructing RSS feeds.

use std::path::PathBuf;

use chrono::{DateTime, NaiveDateTime, Utc};
use rocket::http::uri::Absolute;
use rocket::uri;
use rss::extension::itunes::{
    ITunesCategoryBuilder, ITunesChannelExtensionBuilder, ITunesItemExtensionBuilder,
};
use rss::{
    CategoryBuilder, ChannelBuilder, EnclosureBuilder, GuidBuilder, ImageBuilder, ItemBuilder,
};

use crate::backends::{Channel, Item};
use crate::Config;

/// Constructs a feed as string from a back-end channel using the `rss` crate.
///
/// It requires the backend and configuration to be able to construct download URLs.
pub(crate) fn construct(backend_id: &str, config: &Config, channel: Channel) -> rss::Channel {
    let category = CategoryBuilder::default()
        .name(
            channel
                .categories
                .first()
                .map(Clone::clone)
                .unwrap_or_default(),
        )
        .build();
    let unix_timestamp = NaiveDateTime::from_timestamp_opt(0, 0)
        .expect("Out-of-range seconds or invalid nanoseconds");
    let mut last_build = DateTime::from_utc(unix_timestamp, Utc);
    let generator = String::from(concat!(
        env!("CARGO_PKG_NAME"),
        " ",
        env!("CARGO_PKG_VERSION")
    ));
    let image = channel
        .image
        .clone()
        .map(|url| ImageBuilder::default().link(url.clone()).url(url).build());
    let items = channel
        .items
        .into_iter()
        .map(|item| construct_item(backend_id, config, item, &mut last_build))
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
        .image(channel.image.map(String::from))
        .explicit(Some(String::from("no")))
        .summary(Some(channel.description.clone()))
        .build();

    ChannelBuilder::default()
        .title(channel.title)
        .link(channel.link)
        .description(channel.description)
        .category(category)
        .last_build_date(Some(last_build.to_rfc2822()))
        .generator(Some(generator))
        .image(image)
        .items(items)
        .itunes_ext(Some(itunes_ext))
        .build()
}

/// Constructs an RSS feed item from a back-end item using the `rss` crate.
///
/// It requires the backend and configuration to be able to construct download URLs.
/// It also bumps the last build timestamp if the last updated timestamp is later than the current
/// value.
fn construct_item(
    backend_id: &str,
    config: &Config,
    item: Item,
    last_build: &mut DateTime<Utc>,
) -> rss::Item {
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
        Absolute::parse(&config.public_url).expect("valid URL"),
        crate::get_download(backend_id = backend_id, file = item.enclosure.file)
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
        .image(item.image.map(String::from))
        .duration(item.duration.map(|dur| format!("{dur}")))
        .subtitle(item.description.clone())
        .keywords(Some(keywords))
        .build();

    if item.updated_at > *last_build {
        *last_build = item.updated_at;
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
}
