extern crate dotenv;

use dotenv::dotenv;
use std::env;
use url::Url;

mod imageset;
mod lfm_date;
mod track;
mod types;
use types::*;

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
