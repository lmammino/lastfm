use lastfm::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .api_key("some-api-key")
        .username("loige")
        .reqwest_client(reqwest::Client::new())
        .base_url("http://localhost:8080".parse().unwrap())
        .build();

    // do something with client...
    dbg!(client);

    Ok(())
}
