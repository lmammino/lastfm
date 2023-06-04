//! # Recent tracks page
//!
//! Defines the [`RecentTracksPage`] struct and its methods.
use crate::{error_response::ErrorResponse, track::Track};
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// A Last.fm recent tracks response. Can either be an error or an actual page of recent tracks.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecentTracksResponse {
    Error(ErrorResponse),
    RecentTracksPage(RecentTracksPage),
}

/// A Last.fm recent tracks page.
#[derive(Serialize, Debug, Clone)]
pub struct RecentTracksPage {
    pub total_tracks: u64,
    pub tracks: Vec<Track>,
}

impl<'de> Deserialize<'de> for RecentTracksPage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw_data: Value = Deserialize::deserialize(deserializer)?;

        // deserialize recenttracks,
        let raw_recent_tracks = raw_data
            .get("recenttracks")
            .ok_or_else(|| D::Error::missing_field("recenttracks"))?
            .as_object()
            .ok_or_else(|| D::Error::custom("Field recenttracks is not an object"))?;

        // deserialize total_tracks,
        let total_tracks = raw_recent_tracks
            .get("@attr")
            .ok_or_else(|| D::Error::missing_field("@attr"))?
            .as_object()
            .ok_or_else(|| D::Error::custom("Field @attr is not an object"))?
            .get("total")
            .ok_or_else(|| D::Error::missing_field("total"))?
            .as_str()
            .ok_or_else(|| D::Error::custom("Field total is not a string"))?
            .parse::<u64>()
            .map_err(|e| D::Error::custom(format!("Failed to parse total: {e}")))?;

        // deserialize tracks,
        let tracks = raw_recent_tracks
            .get("track")
            .ok_or_else(|| D::Error::missing_field("track"))?
            .as_array()
            .ok_or_else(|| D::Error::custom("Field track is not an array"))?
            .iter()
            .map(|t| {
                serde_json::from_value::<Track>(t.clone())
                    .map_err(|e| D::Error::custom(format!("Cannot deserialize track: {e}")))
            })
            .collect::<Result<Vec<Track>, D::Error>>()?;

        Ok(RecentTracksPage {
            total_tracks,
            tracks,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_deserializes_a_recent_tracks_page() {
        let json_data = include_str!("fixtures/recent_tracks_page.json");

        let recent_tracks_page: RecentTracksPage = serde_json::from_str(json_data).unwrap();
        insta::assert_debug_snapshot!(recent_tracks_page);
    }
}
