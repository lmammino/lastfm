use crate::{imageset::ImageSet, track::Track};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct RecentTracks {
    pub recenttracks: RecentTracksData,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RecentTracksData {
    pub track: Vec<Track>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Artist {
    pub image: ImageSet,
    pub name: String,
    pub mbid: String,
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Album {
    #[serde(alias = "#text")]
    pub text: String,
    pub mbid: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LfmDate {
    pub uts: String,
    #[serde(alias = "#text")]
    pub text: String,
}
