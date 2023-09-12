//! # Client
//!
//! The main client module for the Last.fm API.
//!
//! This module contains the [`Client`] struct and its methods.
//! It also provides a [`ClientBuilder`] to create a new [`Client`].
use crate::{
    errors::Error,
    recent_tracks_page::{RecentTracksPage, RecentTracksResponse},
    retry_strategy::{JitteredBackoff, RetryStrategy},
    track::{NowPlayingTrack, RecordedTrack, Track},
};
use async_stream::try_stream;
use std::{
    env::{self, VarError},
    fmt::Debug,
    time::Duration,
};
use tokio_stream::Stream;
use typed_builder::TypedBuilder;
use url::Url;

/// The default base URL for the Last.fm API.
pub const DEFAULT_BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";

lazy_static! {
    static ref DEFAULT_CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(10))
        .user_agent(format!(
            "github.com/lmammino/lastfm {}",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .expect("Cannot initialize HTTP client");
}

/// Utility function that masks the API key by replacing all but the first 3 characters with `*`.
fn mask_api_key(api_key: &str) -> String {
    api_key
        .chars()
        .enumerate()
        .map(|(i, c)| match i {
            0..=2 => c,
            _ => '*',
        })
        .collect()
}

/// A client for the Last.fm API.
#[derive(TypedBuilder)]
pub struct Client<A: AsRef<str>, U: AsRef<str>> {
    api_key: A,
    username: U,
    #[builder(default = DEFAULT_CLIENT.clone())]
    reqwest_client: reqwest::Client,
    #[builder(default = DEFAULT_BASE_URL.parse().unwrap())]
    base_url: Url,
    #[builder(default = Box::from(JitteredBackoff::default()))]
    retry_strategy: Box<dyn RetryStrategy>,
}

impl<A: AsRef<str>, U: AsRef<str>> Debug for Client<A, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("api_key", &mask_api_key(self.api_key.as_ref()).as_str())
            .field("username", &self.username.as_ref())
            .field("reqwest_client", &self.reqwest_client)
            .field("base_url", &self.base_url)
            .finish()
    }
}

/// Structs that can be used to get a stream of [`RecordedTrack`]s.
#[non_exhaustive]
pub struct RecentTracksFetcher {
    api_key: String,
    username: String,
    current_page: Vec<RecordedTrack>,
    from: Option<i64>,
    to: Option<i64>,
    /// The total number of tracks available in the stream.
    pub total_tracks: u64,
    reqwest_client: reqwest::Client,
    base_url: Url,
    retry_strategy: Box<dyn RetryStrategy>,
}

impl RecentTracksFetcher {
    fn update_current_page(&mut self, page: RecentTracksPage) {
        let mut current_page: Vec<RecordedTrack> = Vec::with_capacity(200);
        let mut to: Option<i64> = None;

        for track in page.tracks {
            if let Track::Recorded(t) = track {
                to = Some(t.date.timestamp());
                current_page.push(t);
            }
        }

        self.current_page = current_page;
        self.to = to;
    }

    /// Converts the current instance into a stream of [`RecordedTrack`]s.
    pub fn into_stream(mut self) -> impl Stream<Item = Result<RecordedTrack, Error>> {
        let recent_tracks = try_stream! {
            loop {
                match self.current_page.pop() {
                    Some(t) => {
                        yield t;
                    }
                    None => {
                        let next_page = get_page(GetPageOptions {
                            client: &self.reqwest_client,
                            retry_strategy: &*self.retry_strategy,
                            base_url: &self.base_url,
                            api_key: &self.api_key,
                            username: &self.username,
                            limit: 200,
                            from: self.from,
                            to: self.to
                        }).await?;
                        if next_page.tracks.is_empty() {
                            break;
                        }
                        self.update_current_page(next_page);
                    },
                }
            }
        };

        recent_tracks
    }
}

/// Configuration options used for the [`Client::get_page`] function.
struct GetPageOptions<'a> {
    client: &'a reqwest::Client,
    retry_strategy: &'a dyn RetryStrategy,
    base_url: &'a Url,
    api_key: &'a str,
    username: &'a str,
    limit: u32,
    from: Option<i64>,
    to: Option<i64>,
}

/// Gets a page of tracks from the Last.fm API.
async fn get_page(options: GetPageOptions<'_>) -> Result<RecentTracksPage, Error> {
    let mut url_query = vec![
        ("method", "user.getrecenttracks".to_string()),
        ("user", options.username.to_string()),
        ("format", "json".to_string()),
        ("extended", "1".to_string()),
        ("limit", options.limit.to_string()),
        ("api_key", options.api_key.to_string()),
    ];

    if let Some(from) = options.from {
        url_query.push(("from", from.to_string()));
    }

    if let Some(to) = options.to {
        url_query.push(("to", to.to_string()));
    }

    let url = Url::parse_with_params(options.base_url.as_str(), &url_query).unwrap();

    let mut errors: Vec<Error> = Vec::new();
    let mut num_retries: usize = 0;
    while let Some(retry_delay) = options.retry_strategy.should_retry_after(num_retries) {
        let res = options.client.get(&(url).to_string()).send().await;
        match res {
            Ok(res) => {
                let page: RecentTracksResponse = res.json().await?;
                match page {
                    RecentTracksResponse::RecentTracksPage(page) => {
                        return Ok(page);
                    }
                    RecentTracksResponse::Error(e) => {
                        tracing::error!("LastFm Error: {}", e.message);
                        if !e.is_retriable() {
                            return Err(e.into());
                        }
                        tokio::time::sleep(retry_delay).await;
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error: {}", e);
                errors.push(e.into());
                tokio::time::sleep(retry_delay).await;
            }
        }
        num_retries += 1;
    }

    Err(Error::TooManyRetry(errors))
}

impl<A: AsRef<str>, U: AsRef<str>> Client<A, U> {
    /// Creates a new [`Client`] with the given username.
    /// The API key is read from the `LASTFM_API_KEY` environment variable.
    /// This method is a shortcut for [`ClientBuilder::from_env`] but, in case of failure, it will panic rather than returning an error.
    ///
    /// # Panics
    /// If the environment variable is not set, this function will panic.
    pub fn from_env(username: U) -> Client<String, U> {
        Self::try_from_env(username).expect("Missing LASTFM_API_KEY environment variable")
    }

    /// Creates a new [`Client`] with the given username.
    /// The API key is read from the `LASTFM_API_KEY` environment variable.
    /// If the environment variable is not set, this function will return an error.
    pub fn try_from_env(username: U) -> Result<Client<String, U>, VarError> {
        let api_key = env::var("LASTFM_API_KEY")?;
        Ok(Client::builder()
            .username(username)
            .api_key(api_key)
            .build())
    }

    /// Fetches the currently playing track for the user (if any)
    pub async fn now_playing(&self) -> Result<Option<NowPlayingTrack>, Error> {
        let page = get_page(GetPageOptions {
            client: &self.reqwest_client,
            retry_strategy: &*self.retry_strategy,
            base_url: &self.base_url,
            api_key: self.api_key.as_ref(),
            username: self.username.as_ref(),
            limit: 1,
            from: None,
            to: None,
        })
        .await?;

        match page.tracks.first() {
            Some(Track::NowPlaying(t)) => Ok(Some(t.clone())),
            _ => Ok(None),
        }
    }

    /// Creates a new [`RecentTracksFetcher`] that can be used to fetch all of the user's recent tracks.
    pub async fn all_tracks(self) -> Result<RecentTracksFetcher, Error> {
        self.recent_tracks(None, None).await
    }

    /// Creates a new [`RecentTracksFetcher`] that can be used to fetch the user's recent tracks in a given time range.
    ///
    /// The `from` and `to` parameters are Unix timestamps (in seconds).
    pub async fn recent_tracks(
        self,
        from: Option<i64>,
        to: Option<i64>,
    ) -> Result<RecentTracksFetcher, Error> {
        let page = get_page(GetPageOptions {
            client: &self.reqwest_client,
            retry_strategy: &*self.retry_strategy,
            base_url: &self.base_url,
            api_key: self.api_key.as_ref(),
            username: self.username.as_ref(),
            limit: 200,
            from,
            to,
        })
        .await?;

        let mut fetcher = RecentTracksFetcher {
            api_key: self.api_key.as_ref().to_string(),
            username: self.username.as_ref().to_string(),
            current_page: vec![],
            from,
            to,
            total_tracks: page.total_tracks,
            reqwest_client: self.reqwest_client,
            base_url: self.base_url,
            retry_strategy: self.retry_strategy,
        };

        fetcher.update_current_page(page);

        Ok(fetcher)
    }
}
