#[macro_use]
extern crate lazy_static;

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

    let client = Client::from_env("loige");

    let now_playing = client.now_playing().await?;
    if let Some(track) = now_playing {
        println!("Now playing: {} - {}", track.artist.name, track.name);
    }

    let tracks = client.all_tracks().await?;
    println!("Total tracks: {}", tracks.total_tracks);

    let recent_tracks = tracks.into_stream();
    pin_mut!(recent_tracks); // needed for iteration
    while let Some(track) = recent_tracks.next().await {
        match track {
            Ok(track) => {
                println!(
                    "{}: {} - {}",
                    track.date.to_rfc2822(),
                    track.artist.name,
                    track.name
                );
            }
            Err(e) => {
                println!("Error fetching data: {:?}", e);
            }
        }
    }
    Ok(())
}
