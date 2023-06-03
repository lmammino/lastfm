use lastfm::ClientBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::new("some-api-key", "loige")
        .reqwest_client(reqwest::Client::new())
        .base_url("http://localhost:8080".parse().unwrap())
        .build();

    // do something with client...
    dbg!(client);

    Ok(())
}
