use lastfm::ClientBuilder;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("LASTFM_API_KEY")?;

    let client = ClientBuilder::new(api_key, "loige")
        .reqwest_client(reqwest::Client::new())
        .base_url("http://localhost:8080".parse().unwrap())
        .build();

    // do something with client...
    dbg!(client);

    Ok(())
}
