use async_stream::stream;
use std::env;
use tokio_stream::Stream;

use super::track::{NowPlayingTrack, RecordedTrack};

pub struct Client {
    api_key: String,
    username: String,
    now_playing: Option<NowPlayingTrack>,
    current_page: Vec<RecordedTrack>,
    next_timestamp: Option<i64>,
    has_more_pages: bool,
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

    fn load_next_page(&mut self) {
        todo!()
    }

    pub async fn now_playing(&mut self) -> &Option<NowPlayingTrack> {
        self.load_next_page();
        &self.now_playing
    }

    pub fn recent_tracks(&mut self) -> impl Stream<Item = RecordedTrack> {
        let s = stream! {
            yield todo!()
        };

        s
    }
}
