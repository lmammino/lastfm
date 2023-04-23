use lastfm::ClientBuilder;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("LASTFM_API_KEY")?;

    let mut client_builder = ClientBuilder::new(api_key, "loige".to_string());
    client_builder.reqwest_client(reqwest::Client::new());
    client_builder.base_url("http://localhost:8080".parse().unwrap());
    let client = client_builder.build();

    // do something with client...

    Ok(())
}
