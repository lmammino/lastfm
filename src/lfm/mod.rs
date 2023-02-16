pub mod artist;
pub mod client;
pub mod imageset;
pub mod lfm_date;
pub mod track;

use self::track::Track;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RecentTracks {
    pub recenttracks: RecentTracksData,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RecentTracksData {
    pub track: Vec<Track>,
}
