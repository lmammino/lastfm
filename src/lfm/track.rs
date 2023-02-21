use chrono::{DateTime, LocalResult, TimeZone, Utc};
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

use crate::artist::Artist;

use super::imageset::ImageSet;

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Track {
    NowPlaying(NowPlayingTrack),
    Recorded(RecordedTrack),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NowPlayingTrack {
    pub artist: Artist,
    pub name: String,
    pub image: ImageSet,
    pub album: String,
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordedTrack {
    pub artist: Artist,
    pub name: String,
    pub image: ImageSet,
    pub album: String,
    pub url: String,
    pub date: DateTime<Utc>,
}

fn is_now_playing(value: &Map<String, Value>) -> bool {
    let attr = value.get("@attr");
    if attr.is_none() {
        return false;
    }
    let attr = attr.unwrap();
    if !attr.is_object() {
        return false;
    }
    let attr = attr.as_object().unwrap();
    let now_playing = attr.get("nowplaying");
    if now_playing.is_none() {
        return false;
    }
    let now_playing = now_playing.unwrap().as_str();
    if now_playing.is_none() {
        return false;
    }

    now_playing.unwrap() == "true"
}

impl<'de> Deserialize<'de> for Track {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw_data: Value = Deserialize::deserialize(deserializer)?;
        if !raw_data.is_object() {
            return Err(D::Error::custom("Expected Object"));
        }

        let raw_data = raw_data.as_object().unwrap();

        // deserialize Artist,
        let raw_artist = raw_data
            .get("artist")
            .ok_or_else(|| D::Error::missing_field("artist"))?;
        let artist: Artist = serde_json::from_value::<Artist>(raw_artist.clone())
            .map_err(|e| D::Error::custom(format!("Cannot deserialize artist: {e}")))?;

        // deserialize name,
        let name = raw_data
            .get("name")
            .ok_or_else(|| D::Error::missing_field("name"))?
            .as_str()
            .ok_or_else(|| D::Error::custom("Field name is not a string"))?;

        // deserialize image
        let raw_image = raw_data
            .get("image")
            .ok_or_else(|| D::Error::missing_field("image"))?;
        let image: ImageSet = serde_json::from_value::<ImageSet>(raw_image.clone())
            .map_err(|e| D::Error::custom(format!("Cannot deserialize image: {e}")))?;

        // deserialize album
        let album = raw_data
            .get("album")
            .ok_or_else(|| D::Error::missing_field("album"))?
            .as_object()
            .ok_or_else(|| D::Error::custom("Field album is not an object"))?
            .get("#text")
            .ok_or_else(|| D::Error::missing_field("#text"))?
            .as_str()
            .ok_or_else(|| D::Error::custom("Field #text is not a string"))?;

        // deserialize url
        let url = raw_data
            .get("url")
            .ok_or_else(|| D::Error::missing_field("url"))?
            .as_str()
            .ok_or_else(|| D::Error::custom("Field url is not a string"))?;

        if is_now_playing(raw_data) {
            return Ok(Track::NowPlaying(NowPlayingTrack {
                artist,
                name: name.to_string(),
                image,
                album: album.to_string(),
                url: url.to_string(),
            }));
        }

        // deserialize date
        let uts = raw_data
            .get("date")
            .ok_or_else(|| D::Error::missing_field("date"))?
            .as_object()
            .ok_or_else(|| D::Error::custom("Field date is not an object"))?
            .get("uts")
            .ok_or_else(|| D::Error::missing_field("uts"))?
            .as_str()
            .ok_or_else(|| D::Error::custom("Field uts is not a string"))?
            .parse::<i64>()
            .map_err(|_| D::Error::custom("Failed to parse uts as i64"))?;

        let local_result = Utc.timestamp_opt(uts, 0);

        if let LocalResult::Single(date) = local_result {
            Ok(Track::Recorded(RecordedTrack {
                artist,
                name: name.to_string(),
                image,
                album: album.to_string(),
                url: url.to_string(),
                date,
            }))
        } else {
            Err(D::Error::custom("Failed to parse uts as i64"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_playing() {
        let now_playing = include_str!("fixtures/now_playing_track.json");
        let track: Track = serde_json::from_str(now_playing).unwrap();
        insta::assert_debug_snapshot!(track);
    }

    #[test]
    fn test_recorded() {
        let now_playing = include_str!("fixtures/recorded_track.json");

        let track: Track = serde_json::from_str(now_playing).unwrap();
        insta::assert_debug_snapshot!(track);
    }
}
