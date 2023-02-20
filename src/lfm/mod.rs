pub mod artist;
pub mod client;
pub mod imageset;
pub mod lfm_date;
pub mod track;

use self::track::Track;
use serde::{Deserialize, Serialize};

// TODO: capture metadata as follows:
/*
Object {
    "recenttracks": Object {
        "@attr": Object {
            "page": String("1"),
            "perPage": String("50"),
            "total": String("290827"),
            "totalPages": String("5817"),
            "user": String("loige"),
        },
*/
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RecentTracks {
    pub recenttracks: RecentTracksData,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RecentTracksData {
    pub track: Vec<Track>,
}
