extern crate dotenv;

use dotenv::dotenv;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;

mod lfm;
use lfm::*;

use crate::lfm::client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let mut client = Client::from_env("loige");

    let info = client.get_info().await;
    println!("Total tracks: {}", info.total_tracks);
    if let Some(track) = info.now_playing {
        println!("Now playing: {} - {}", track.artist.name, track.name);
    }

    let recent_tracks = client.recent_tracks();
    pin_mut!(recent_tracks); // needed for iteration
    while let Some(track) = recent_tracks.next().await {
        println!(
            "{}: {} - {}",
            track.date.to_rfc2822(),
            track.artist.name,
            track.name
        );
    }
    Ok(())
}
