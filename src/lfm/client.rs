use async_stream::stream;
use std::env;
use tokio_stream::Stream;
use url::Url;

use super::{
    recent_tracks_page::RecentTracksPage,
    track::{NowPlayingTrack, RecordedTrack, Track},
};

const BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";

pub struct Client {
    api_key: String,
    username: String,
    now_playing: Option<NowPlayingTrack>,
    current_page: Vec<RecordedTrack>,
    next_timestamp: Option<i64>,
    has_loaded_first_page: bool,
    total_tracks: u64,
}

pub struct ClientInfo<'a> {
    pub total_tracks: &'a u64,
    pub now_playing: &'a Option<NowPlayingTrack>,
}

impl Client {
    pub fn new<A: AsRef<str>, U: AsRef<str>>(api_key: A, username: U) -> Self {
        Client {
            api_key: api_key.as_ref().to_string(),
            username: username.as_ref().to_string(),
            now_playing: None,
            current_page: Vec::with_capacity(200),
            next_timestamp: None,
            has_loaded_first_page: false,
            total_tracks: 0,
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
                ("limit", "200"),
                ("api_key", self.api_key.as_str()),
            ],
        )?;

        if let Some(timestamp) = self.next_timestamp {
            url.query_pairs_mut()
                .append_pair("to", &timestamp.to_string());
        }

        let page: RecentTracksPage = reqwest::get(url.to_string()).await?.json().await?;

        if !self.has_loaded_first_page {
            self.has_loaded_first_page = true;
            self.total_tracks = page.total_tracks;
        }

        self.current_page.clear();

        for track in page.tracks {
            match track {
                Track::Recorded(t) => {
                    self.next_timestamp = Some(t.date.timestamp());
                    self.current_page.push(t);
                }
                Track::NowPlaying(t) => {
                    self.now_playing = Some(t);
                }
            }
        }

        Ok(self.current_page.is_empty())
    }

    // TODO: how to handle errors?!
    pub async fn get_info(&mut self) -> ClientInfo {
        if !self.has_loaded_first_page {
            self.load_next_page().await.unwrap();
        }

        ClientInfo {
            total_tracks: &self.total_tracks,
            now_playing: &self.now_playing,
        }
    }

    // TODO: how to handle errors?!
    pub fn recent_tracks(mut self) -> impl Stream<Item = RecordedTrack> {
        let recent_tracks = stream! {
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

        recent_tracks
    }
}
