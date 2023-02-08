extern crate dotenv;

use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
struct RecentTracks {
    recenttracks: RecentTracksData,
}

#[derive(Deserialize, Serialize, Debug)]
struct RecentTracksData {
    track: Vec<Track>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Track {
    artist: Artist,
    album: Album,
    name: String,
    date: LfmDate,
    url: String,
    mbid: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Artist {
    image: Vec<Image>,
    name: String,
    mbid: String,
    url: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Album {
    #[serde(alias = "#text")]
    text: String,
    mbid: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Image {
    #[serde(alias = "#text")]
    text: String,
    size: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct LfmDate {
    uts: String,
    #[serde(alias = "#text")]
    text: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let lastfm_api_key = env::var("LASTFM_API_KEY").expect("LASTFM_API_KEY must be set");

    let base_url = "https://ws.audioscrobbler.com/2.0/";

    let url = Url::parse_with_params(
        base_url,
        &[
            ("method", "user.getrecenttracks"),
            ("user", "loige"),
            ("format", "json"),
            ("extended", "1"),
            ("api_key", &lastfm_api_key),
        ],
    )?;

    let resp: RecentTracks = reqwest::get(url.to_string()).await?.json().await?;
    let last_song = resp.recenttracks.track.first().unwrap();
    println!("{last_song:#?}");
    Ok(())
}
