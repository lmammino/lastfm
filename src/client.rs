use super::{
    errors::Error,
    recent_tracks_page::RecentTracksPage,
    track::{NowPlayingTrack, RecordedTrack, Track},
};
use crate::{recent_tracks_page::RecentTracksResponse, retry_delay::RetryDelay};
use async_stream::try_stream;
use std::{
    env::{self, VarError},
    time::Duration,
};
use tokio_stream::Stream;
use url::Url;

const BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";

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

pub struct ClientBuilder<A: AsRef<str>, U: AsRef<str>> {
    api_key: A,
    username: U,
    reqwest_client: Option<reqwest::Client>,
    base_url: Option<Url>,
}

impl<A: AsRef<str>, U: AsRef<str>> ClientBuilder<A, U> {
    pub fn new(api_key: A, username: U) -> Self {
        Self {
            api_key,
            username,
            reqwest_client: None,
            base_url: None,
        }
    }

    pub fn reqwest_client(&mut self, client: reqwest::Client) -> &mut Self {
        self.reqwest_client = Some(client);
        self
    }

    pub fn base_url(&mut self, base_url: Url) -> &mut Self {
        self.base_url = Some(base_url);
        self
    }

    pub fn build(self) -> Client {
        Client {
            api_key: self.api_key.as_ref().to_string(),
            username: self.username.as_ref().to_string(),
            reqwest_client: self
                .reqwest_client
                .unwrap_or_else(|| DEFAULT_CLIENT.clone()),
            base_url: self.base_url.unwrap_or_else(|| BASE_URL.parse().unwrap()),
        }
    }
}

pub struct Client {
    api_key: String,
    username: String,
    reqwest_client: reqwest::Client,
    base_url: Url,
}

pub struct RecentTracksFetcher {
    api_key: String,
    username: String,
    current_page: Vec<RecordedTrack>,
    from: Option<i64>,
    to: Option<i64>,
    pub total_tracks: u64,
    reqwest_client: reqwest::Client,
    base_url: Url,
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

    pub fn into_stream(mut self) -> impl Stream<Item = Result<RecordedTrack, Error>> {
        let recent_tracks = try_stream! {
            loop {
                match self.current_page.pop() {
                    Some(t) => {
                        yield t;
                    }
                    None => {
                        let next_page = get_page(&self.reqwest_client, &self.base_url, &self.api_key, &self.username, 200, self.from, self.to).await?;
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

async fn get_page<A: AsRef<str>, U: AsRef<str>>(
    client: &reqwest::Client,
    base_url: &Url,
    api_key: A,
    username: U,
    limit: u32,
    from: Option<i64>,
    to: Option<i64>,
) -> Result<RecentTracksPage, Error> {
    let mut url_query = vec![
        ("method", "user.getrecenttracks".to_string()),
        ("user", username.as_ref().to_string()),
        ("format", "json".to_string()),
        ("extended", "1".to_string()),
        ("limit", limit.to_string()),
        ("api_key", api_key.as_ref().to_string()),
    ];

    if let Some(from) = from {
        url_query.push(("from", from.to_string()));
    }

    if let Some(to) = to {
        url_query.push(("to", to.to_string()));
    }

    let url = Url::parse_with_params(base_url.as_str(), &url_query).unwrap();

    let retry = RetryDelay::default();
    let mut errors: Vec<Error> = Vec::new();
    for sleep_time in retry {
        let res = client.get(&(url).to_string()).send().await;
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
                        tokio::time::sleep(sleep_time).await;
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error: {}", e);
                errors.push(e.into());
                tokio::time::sleep(sleep_time).await;
            }
        }
    }

    Err(Error::TooManyRetry(errors))
}

impl Client {
    pub fn from_env<U: AsRef<str>>(username: U) -> Self {
        Self::try_from_env(username).unwrap()
    }

    pub fn try_from_env<U: AsRef<str>>(username: U) -> Result<Self, VarError> {
        let api_key = env::var("LASTFM_API_KEY")?;
        Ok(ClientBuilder::new(api_key, username).build())
    }

    pub async fn now_playing(&self) -> Result<Option<NowPlayingTrack>, Error> {
        let page = get_page(
            &self.reqwest_client,
            &self.base_url,
            &self.api_key,
            &self.username,
            1,
            None,
            None,
        )
        .await?;

        match page.tracks.first() {
            Some(Track::NowPlaying(t)) => Ok(Some(t.clone())),
            _ => Ok(None),
        }
    }

    pub async fn all_tracks(self) -> Result<RecentTracksFetcher, Error> {
        self.recent_tracks(None, None).await
    }

    pub async fn recent_tracks(
        self,
        from: Option<i64>,
        to: Option<i64>,
    ) -> Result<RecentTracksFetcher, Error> {
        let page = get_page(
            &self.reqwest_client,
            &self.base_url,
            &self.api_key,
            &self.username,
            200,
            from,
            to,
        )
        .await?;

        let mut fetcher = RecentTracksFetcher {
            api_key: self.api_key.clone(),
            username: self.username.clone(),
            current_page: vec![],
            from,
            to,
            total_tracks: page.total_tracks,
            reqwest_client: self.reqwest_client,
            base_url: self.base_url,
        };

        fetcher.update_current_page(page);

        Ok(fetcher)
    }
}
