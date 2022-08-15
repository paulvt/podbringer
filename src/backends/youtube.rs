//! The YouTube back-end.
//!
//! It uses the `ytextract` crate to retrieve the feed (channel or playlist) and items (videos).

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use cached::proc_macro::cached;
use chrono::Utc;
use reqwest::Url;
use rocket::futures::StreamExt;
use ytextract::playlist::video::{Error as YouTubeVideoError, Video as YouTubeVideo};
use ytextract::{
    Channel as YouTubeChannel, Client, Playlist as YouTubePlaylist, Stream as YouTubeStream,
};

use super::{Channel, Enclosure, Item};
use crate::{Error, Result};

/// The base URL for YouTube channels.
const CHANNEL_BASE_URL: &str = "https://www.youtube.com/channel";

/// The base URL for YouTube playlists.
const PLAYLIST_BASE_URL: &str = "https://www.youtube.com/channel";

/// The base URL for YouTube videos.
const VIDEO_BASE_URL: &str = "https://www.youtube.com/watch";

/// Creates a YouTube back-end.
pub(crate) fn backend() -> Backend {
    Backend::new()
}

/// The YouTube back-end.
pub struct Backend {
    /// The client capable of interacting with YouTube.
    client: Client,
}

impl Backend {
    /// Creates a new YouTube back-end.
    fn new() -> Self {
        let client = Client::new();

        Self { client }
    }
}

#[async_trait]
impl super::Backend for Backend {
    fn name(&self) -> &'static str {
        "YouTube"
    }

    async fn channel(&self, channel_id: &str, item_limit: Option<usize>) -> Result<Channel> {
        // We assume it is a YouTube playlist ID if the channel ID starts with
        // "PL"/"OLAK"/"RDCLAK"; it is considered to be a YouTube channel ID otherwise.
        if channel_id.starts_with("PL")
            || channel_id.starts_with("OLAK")
            || channel_id.starts_with("RDCLAK")
        {
            let (yt_playlist, yt_videos_w_streams) =
                fetch_playlist_videos(&self.client, channel_id, item_limit).await?;

            Ok(Channel::from(YouTubePlaylistWithVideos(
                yt_playlist,
                yt_videos_w_streams,
            )))
        } else {
            let (yt_channel, yt_videos_w_streams) =
                fetch_channel_videos(&self.client, channel_id, item_limit).await?;

            Ok(Channel::from(YouTubeChannelWithVideos(
                yt_channel,
                yt_videos_w_streams,
            )))
        }
    }

    async fn redirect_url(&self, file: &Path) -> Result<String> {
        let id_part = file.with_extension("");
        let video_id = id_part.to_string_lossy();

        retrieve_redirect_url(&self.client, &video_id).await
    }
}

/// A YouTube playlist with its videos.
#[derive(Clone, Debug)]
pub(crate) struct YouTubePlaylistWithVideos(YouTubePlaylist, Vec<YouTubeVideoWithStream>);

/// A YouTube channel with its videos.
#[derive(Clone, Debug)]
pub(crate) struct YouTubeChannelWithVideos(YouTubeChannel, Vec<YouTubeVideoWithStream>);

/// A YouTube video with its stream.
#[derive(Clone, Debug)]
struct YouTubeVideoWithStream {
    /// The information of the YouTube video.
    video: YouTubeVideo,

    /// The metadata of the selected YouTube stream.
    stream: YouTubeStream,

    /// The content of the selected YouTube stream.
    content_length: u64,
}

impl From<YouTubeChannelWithVideos> for Channel {
    fn from(
        YouTubeChannelWithVideos(yt_channel, yt_videos_w_streams): YouTubeChannelWithVideos,
    ) -> Self {
        let mut link = Url::parse(CHANNEL_BASE_URL).expect("valid URL");
        link.path_segments_mut()
            .expect("valid URL")
            .push(&yt_channel.id());
        let author = Some(yt_channel.name().to_string());
        let categories = Vec::from([String::from("Video")]);
        let image = yt_channel
            .avatar()
            .max_by_key(|av| av.width * av.height)
            .map(|av| av.url.clone());
        let items = yt_videos_w_streams.into_iter().map(Item::from).collect();

        Channel {
            title: format!("{0} (via YouTube)", yt_channel.name()),
            link,
            description: yt_channel.description().to_string(),
            author,
            categories,
            image,
            items,
        }
    }
}

impl From<YouTubePlaylistWithVideos> for Channel {
    fn from(
        YouTubePlaylistWithVideos(yt_playlist, yt_videos_w_streams): YouTubePlaylistWithVideos,
    ) -> Self {
        let mut link = Url::parse(PLAYLIST_BASE_URL).expect("valid URL");
        link.query_pairs_mut()
            .append_pair("list", &yt_playlist.id().to_string());
        let author = yt_playlist.channel().map(|chan| chan.name().to_string());
        // FIXME: Don't hardcode the category!
        let categories = Vec::from([String::from("Video")]);
        let image = yt_playlist
            .thumbnails()
            .iter()
            .max_by_key(|tn| tn.width * tn.height)
            .map(|tn| tn.url.clone());
        let items = yt_videos_w_streams.into_iter().map(Item::from).collect();

        Channel {
            title: format!("{0} (via YouTube)", yt_playlist.title()),
            link,
            description: yt_playlist.description().to_string(),
            author,
            categories,
            image,
            items,
        }
    }
}

impl From<YouTubeVideoWithStream> for Item {
    fn from(
        YouTubeVideoWithStream {
            video,
            stream,
            content_length: length,
        }: YouTubeVideoWithStream,
    ) -> Self {
        let id = video.id().to_string();

        let mime_type = stream.mime_type().to_string();
        // Ignore everything from MIME type parameter seperator on for extension look-up.
        let mime_sep = mime_type.find(';').unwrap_or(mime_type.len());
        let extension = mime_db::extension(&mime_type[..mime_sep]).unwrap_or_default();
        let file = PathBuf::from(&id).with_extension(extension);
        let enclosure = Enclosure {
            file,
            mime_type,
            length,
        };

        let mut link = Url::parse(VIDEO_BASE_URL).expect("valid URL");
        link.query_pairs_mut().append_pair("v", &id);
        let description = Some(format!("Taken from YouTube: {0}", link));
        let duration = Some(video.length().as_secs() as u32);
        let image = video
            .thumbnails()
            .iter()
            .max_by_key(|tn| tn.width * tn.height)
            .map(|tn| tn.url.clone());

        Item {
            title: video.title().to_string(),
            link,
            description,
            categories: Default::default(),
            enclosure,
            duration,
            guid: id,
            keywords: Default::default(),
            image,
            updated_at: Utc::now(), // TODO: Get a decent timestamp somewhere?!
        }
    }
}

/// Fetches the YouTube playlist videos for the given ID.
///
/// If the result is [`Ok`], the playlist will be cached for 24 hours for the given playlist ID.
#[cached(
    key = "(String, Option<usize>)",
    convert = r#"{ (playlist_id.to_owned(), item_limit) }"#,
    time = 86400,
    result = true
)]
async fn fetch_playlist_videos(
    client: &Client,
    playlist_id: &str,
    item_limit: Option<usize>,
) -> Result<(YouTubePlaylist, Vec<YouTubeVideoWithStream>)> {
    let id = playlist_id.parse()?;
    let yt_playlist = client.playlist(id).await?;
    let yt_videos_w_streams: Vec<_> = match item_limit {
        Some(n) => {
            yt_playlist
                .videos()
                .filter_map(fetch_stream)
                .take(n)
                .collect()
                .await
        }
        None => {
            yt_playlist
                .videos()
                .filter_map(fetch_stream)
                .collect()
                .await
        }
    };

    Ok((yt_playlist, yt_videos_w_streams))
}

/// Fetches the YouTube channel videos for the given ID.
#[cached(
    key = "(String, Option<usize>)",
    convert = r#"{ (channel_id.to_owned(), item_limit) }"#,
    time = 86400,
    result = true
)]
async fn fetch_channel_videos(
    client: &Client,
    channel_id: &str,
    item_limit: Option<usize>,
) -> Result<(YouTubeChannel, Vec<YouTubeVideoWithStream>)> {
    let id = channel_id.parse()?;
    let yt_channel = client.channel(id).await?;
    let yt_videos_w_streams: Vec<_> = match item_limit {
        Some(n) => {
            yt_channel
                .uploads()
                .await?
                .take(n)
                .filter_map(fetch_stream)
                .collect()
                .await
        }
        None => {
            yt_channel
                .uploads()
                .await?
                .filter_map(fetch_stream)
                .collect()
                .await
        }
    };

    Ok((yt_channel, yt_videos_w_streams))
}

/// Fetches the stream and relevant metadata for a YouTube video result.
///
/// If there is a video retieving the metadata, the video is discarded/ignored.
/// If there are problems retrieving the streams or metadata, the video is also discarded.
async fn fetch_stream(
    yt_video: Result<YouTubeVideo, YouTubeVideoError>,
) -> Option<YouTubeVideoWithStream> {
    match yt_video {
        Ok(video) => {
            let stream = video
                .streams()
                .await
                .ok()?
                .filter(|v| v.is_audio())
                .max_by_key(|v| v.bitrate())?;
            let content_length = stream.content_length().await.ok()?;

            Some(YouTubeVideoWithStream {
                video,
                stream,
                content_length,
            })
        }
        Err(_) => None,
    }
}

/// Retrieves the redirect URL for the provided YouTube video ID.
///
/// If the result is [`Ok`], the redirect URL will be cached for 24 hours for the given video ID.
#[cached(
    key = "String",
    convert = r#"{ video_id.to_owned() }"#,
    time = 86400,
    result = true
)]
async fn retrieve_redirect_url(client: &Client, video_id: &str) -> Result<String> {
    let video_id = video_id.parse()?;
    let video = client.video(video_id).await?;
    let stream = video
        .streams()
        .await?
        .filter(|v| v.is_audio())
        .max_by_key(|v| v.bitrate())
        .ok_or(Error::NoRedirectUrlFound)?;

    Ok(stream.url().to_string())
}
