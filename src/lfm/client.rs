use crate::lfm::{recent_tracks_page::RecentTracksResponse, retry_delay::RetryDelay};

use super::{
    recent_tracks_page::RecentTracksPage,
    track::{NowPlayingTrack, RecordedTrack, Track},
};
use async_stream::try_stream;
use std::{env, time::Duration};
use tokio_stream::Stream;
use url::Url;

const BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(10))
        .user_agent(format!(
            "github.com/lmammino/rust-lastfm {}",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .expect("Cannot initialize HTTP client");
}

pub struct Client {
    api_key: String,
    username: String,
}

pub struct RecentTracksFetcher {
    api_key: String,
    username: String,
    current_page: Vec<RecordedTrack>,
    from: Option<i64>,
    to: Option<i64>,
    pub total_tracks: u64,
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

    pub fn into_stream(
        mut self,
    ) -> impl Stream<Item = Result<RecordedTrack, Box<dyn std::error::Error>>> {
        let recent_tracks = try_stream! {
            loop {
                match self.current_page.pop() {
                    Some(t) => {
                        yield t;
                    }
                    None => {
                        let next_page = get_page(&self.api_key, &self.username, 200, self.from, self.to).await?;
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
    api_key: A,
    username: U,
    limit: u32,
    from: Option<i64>,
    to: Option<i64>,
) -> Result<RecentTracksPage, Box<dyn std::error::Error>> {
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

    let url = Url::parse_with_params(BASE_URL, &url_query).unwrap();

    let retry = RetryDelay::new(5);
    for sleep_time in retry {
        let res = CLIENT.get(&(url).to_string()).send().await;
        match res {
            Ok(res) => {
                let page: RecentTracksResponse = res.json().await?;
                match page {
                    RecentTracksResponse::RecentTracksPage(page) => {
                        return Ok(page);
                    }
                    RecentTracksResponse::Error(e) => {
                        println!("Error: {}", e.message);
                        tokio::time::sleep(sleep_time).await;
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                tokio::time::sleep(sleep_time).await;
            }
        }
    }

    Err("Too many retry".into())
}

impl Client {
    pub fn new<A: AsRef<str>, U: AsRef<str>>(api_key: A, username: U) -> Self {
        Client {
            api_key: api_key.as_ref().to_string(),
            username: username.as_ref().to_string(),
        }
    }

    pub fn from_env<U: AsRef<str>>(username: U) -> Self {
        let api_key = env::var("LASTFM_API_KEY").expect("LASTFM_API_KEY not set");
        Client::new(api_key, username)
    }

    pub async fn now_playing(&self) -> Result<Option<NowPlayingTrack>, Box<dyn std::error::Error>> {
        let page = get_page(&self.api_key, &self.username, 1, None, None).await?;

        match page.tracks.first() {
            Some(Track::NowPlaying(t)) => Ok(Some(t.clone())),
            _ => Ok(None),
        }
    }

    pub async fn all_tracks(self) -> Result<RecentTracksFetcher, Box<dyn std::error::Error>> {
        self.recent_tracks(None, None).await
    }

    pub async fn recent_tracks(
        self,
        from: Option<i64>,
        to: Option<i64>,
    ) -> Result<RecentTracksFetcher, Box<dyn std::error::Error>> {
        let page = get_page(&self.api_key, &self.username, 200, from, to).await?;

        let mut fetcher = RecentTracksFetcher {
            api_key: self.api_key.clone(),
            username: self.username.clone(),
            current_page: vec![],
            from,
            to,
            total_tracks: page.total_tracks,
        };

        fetcher.update_current_page(page);

        Ok(fetcher)
    }
}
