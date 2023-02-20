use async_stream::stream;
use std::env;
use tokio_stream::Stream;
use url::Url;

use super::{
    track::{NowPlayingTrack, RecordedTrack, Track},
    RecentTracks,
};

const BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";

pub struct Client {
    api_key: String,
    username: String,
    now_playing: Option<NowPlayingTrack>,
    current_page: Vec<RecordedTrack>,
    next_timestamp: Option<i64>,
    has_more_pages: bool, // TODO: do we need these?
    total_tracks: u64,
    retrieved_tracks: u64,
}

impl Client {
    pub fn new<A: AsRef<str>, U: AsRef<str>>(api_key: A, username: U) -> Self {
        Client {
            api_key: api_key.as_ref().to_string(),
            username: username.as_ref().to_string(),
            now_playing: None,
            current_page: Vec::with_capacity(50),
            next_timestamp: None,
            has_more_pages: true,
            total_tracks: 0,
            retrieved_tracks: 0,
        }
    }

    pub fn from_env<U: AsRef<str>>(username: U) -> Self {
        let api_key = env::var("LASTFM_API_KEY").expect("LASTFM_API_KEY not set");
        Client::new(api_key, username)
    }

    // TODO: create proper error types for this client
    async fn load_next_page(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let mut url = Url::parse_with_params(
            BASE_URL,
            &[
                ("method", "user.getrecenttracks"),
                ("user", self.username.as_str()),
                ("format", "json"),
                ("extended", "1"),
                ("api_key", self.api_key.as_str()),
            ],
        )?;

        if let Some(timestamp) = self.next_timestamp {
            url.query_pairs_mut()
                .append_pair("to", &timestamp.to_string());
        }

        let resp: RecentTracks = reqwest::get(url.to_string()).await?.json().await?;

        self.current_page.clear();
        resp.recenttracks.track.iter().for_each(|t| match t {
            Track::Recorded(t) => {
                // self.total_tracks = t.total;
                self.current_page.push(t.clone());
                self.next_timestamp = Some(t.date.timestamp());
            }
            Track::NowPlaying(t) => {
                self.now_playing = Some(t.clone());
            }
        });

        Ok(self.current_page.is_empty())
    }

    // TODO: how to handle errors?!
    pub async fn now_playing(&mut self) -> &Option<NowPlayingTrack> {
        self.load_next_page().await.unwrap();
        &self.now_playing
    }

    pub fn recent_tracks(&mut self) -> impl Stream<Item = RecordedTrack> + '_ {
        let s = stream! {
            loop {
                match self.current_page.pop() {
                    Some(t) => {
                        yield t;
                    }
                    None => {
                        let is_over = self.load_next_page().await.unwrap();
                        if is_over {
                            break;
                        }
                    },
                }
            }
        };

        s
    }
}
