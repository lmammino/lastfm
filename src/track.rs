use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    lfm_date::LfmDate,
    types::{Album, Artist},
};

#[derive(Deserialize, Serialize, Debug)]
pub struct Track {
    pub artist: Artist,
    pub album: Album,
    pub name: String,
    pub date: Option<LfmDate>,
    pub url: String,
    pub mbid: String,
    #[serde(deserialize_with = "de_now_playing", rename = "@attr", default)]
    pub now_playing: bool,
}

fn de_now_playing<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let attrs: HashMap<String, String> = Deserialize::deserialize(deserializer)?;
    Ok(attrs.get("nowplaying").unwrap_or(&"false".to_string()) == "true")
}

// TODO: write test
