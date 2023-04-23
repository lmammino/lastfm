# rust-lastfm

[![Build Status](https://github.com/lmammino/lastfm/actions/workflows/rust.yml/badge.svg)](https://github.com/lmammino/lastfm/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/lastfm.svg)](https://crates.io/crates/lastfm)
[![docs.rs](https://docs.rs/lastfm/badge.svg)](https://docs.rs/lastfm)



<!-- cargo-sync-readme start -->

`lastfm` is an async Rust client to fetch your [Last.fm](https://last.fm) listening history or the track you are currently playing

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
lastfm = "*"
```

Replace the `*` with the actual version you want to use.


Alternatively you can run:

```bash
cargo add lastfm
````

## Usage

To use this library you will need a Last.fm account and an API key.
You can get one by [registering an application](https://www.last.fm/api/account/create).
If you have already registered an application, you can find your API key in the [API settings](https://www.last.fm/api/accounts).

### Create a new client

If you have your API key exposed through the `LASTFM_API_KEY` environment variable, you can use the `from_env` method:

```rust,no_run
let client = Client::from_env("YOUR_USERNAME");
```

Note: this method will panic if `LASTFM_API_KEY` is not set.

Alternatively, you can use `try_from_env` which will return a `Result`.

```rust,no_run
let maybe_client = Client::try_from_env("YOUR_USERNAME");
match maybe_client {
  Ok(client) => {
    // do something with the client
  }
  Err(e) => {
    // handle error
  }
}
```

Finally, for more advanced configurations you can use a `ClientBuilder`:

```rust
let client = ClientBuilder::new("YOUR_API_KEY", "YOUR_USERNAME").build();
```

### Fetch the track you are currently playing

```rust,no_run
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let client = ClientBuilder::new("YOUR_API_KEY", "YOUR_USERNAME").build();
  let now_playing = client.now_playing().await?;
  if let Some(track) = now_playing {
    println!("Now playing: {} - {}", track.artist.name, track.name);
  }

  Ok(())
}
```

### Fetch your listening history

**Note**: You will need the `futures-util` crate to use the `Stream` returned by `all_tracks`.


```rust,no_run
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let client = ClientBuilder::new("YOUR_API_KEY", "YOUR_USERNAME").build();
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
```

<!-- cargo-sync-readme end -->

## Examples

This package provides some usage examples in the [`examples`](/examples/) folder.

You will need an API key to run the examples so you will need to:

- copy `.env~sample` into `.env`
- add your last.fm API Key in there
- run a give example. E.g.: `cargo run --example fetch_all`


## Other implementations

This project is a port of something I have already done in JavaScript (Node.js). Check out [`lmammino/scrobbles`](https://github.com/lmammino/scrobbles) if you are curious.


## Contributing

Everyone is very welcome to contribute to this project.
You can contribute just by submitting bugs or suggesting improvements by
[opening an issue on GitHub](https://github.com/lmammino/lastfm/issues).


## License

Licensed under [MIT License](LICENSE). © Luciano Mammino.

